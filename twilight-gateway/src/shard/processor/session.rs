use super::{
    super::{json, stage::Stage},
    heartbeat::{Heartbeater, Heartbeats},
};
use leaky_bucket_lite::LeakyBucket;
use serde::ser::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    sync::{
        atomic::{AtomicU64, AtomicU8, Ordering},
        Arc, Mutex as MutexSync,
    },
    time::Duration,
};
use tokio::{
    sync::{
        mpsc::{error::SendError, UnboundedSender},
        OnceCell,
    },
    task::JoinHandle,
};
use tokio_tungstenite::tungstenite::{protocol::CloseFrame, Message as TungsteniteMessage};
use twilight_model::gateway::payload::outgoing::Heartbeat;

// Interval of how often the ratelimit bucket resets, in milliseconds.
const RESET_DURATION_MILLISECONDS: u64 = 60_000;

#[derive(Debug)]
pub struct SessionSendError {
    pub(super) source: Option<Box<dyn Error + Send + Sync>>,
    pub(super) kind: SessionSendErrorType,
}

impl SessionSendError {
    /// Immutable reference to the type of error that occurred.
    pub const fn kind(&self) -> &SessionSendErrorType {
        &self.kind
    }
}

impl Display for SessionSendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            SessionSendErrorType::Serializing => f.write_str("failed to serialize payload as json"),
            SessionSendErrorType::Sending => f.write_str("failed to send message over websocket"),
        }
    }
}

impl Error for SessionSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`SessionSendError`] that occurred.
#[derive(Debug)]
pub enum SessionSendErrorType {
    Sending,
    Serializing,
}

#[derive(Debug)]
pub struct Session {
    pub heartbeater_handle: MutexSync<Option<JoinHandle<()>>>,
    pub heartbeats: Arc<Heartbeats>,
    pub heartbeat_interval: AtomicU64,
    pub id: MutexSync<Option<Box<str>>>,
    pub seq: Arc<AtomicU64>,
    pub stage: AtomicU8,
    pub tx: UnboundedSender<TungsteniteMessage>,
    pub ratelimit: OnceCell<Option<LeakyBucket>>,
}

impl Session {
    pub fn new(tx: UnboundedSender<TungsteniteMessage>, ratelimit_payloads: bool) -> Self {
        let session = Self {
            heartbeater_handle: MutexSync::new(None),
            heartbeats: Arc::new(Heartbeats::default()),
            heartbeat_interval: AtomicU64::new(0),
            id: MutexSync::new(None),
            seq: Arc::new(AtomicU64::new(0)),
            stage: AtomicU8::new(Stage::default() as u8),
            tx,
            ratelimit: OnceCell::new(),
        };

        if !ratelimit_payloads {
            session.disable_ratelimiter();
        }

        session
    }

    /// Sends a payload as a message over the socket.
    ///
    /// # Errors
    ///
    /// Returns a [`SessionSendErrorType::Serializing`] error type when there is
    /// an error serializing the payload into an acceptable format.
    ///
    /// Returns a [`SessionSendErrorType::Sending`] error type when the
    /// receiving channel has hung up. This will only happen when the shard has
    /// either not started or has already shutdown.
    pub fn send(&self, payload: impl Serialize) -> Result<(), SessionSendError> {
        let bytes = json::to_vec(&payload).map_err(|source| SessionSendError {
            kind: SessionSendErrorType::Serializing,
            source: Some(Box::new(source)),
        })?;

        self.tx
            .send(TungsteniteMessage::Binary(bytes))
            .map_err(|source| SessionSendError {
                kind: SessionSendErrorType::Sending,
                source: Some(Box::new(source)),
            })?;

        Ok(())
    }

    pub fn close(
        &self,
        close_frame: Option<CloseFrame<'static>>,
    ) -> Result<(), SendError<TungsteniteMessage>> {
        self.tx.send(TungsteniteMessage::Close(close_frame))
    }

    fn disable_ratelimiter(&self) {
        let _result = self.ratelimit.set(None);
    }

    pub fn heartbeat_interval(&self) -> u64 {
        self.heartbeat_interval.load(Ordering::Relaxed)
    }

    pub fn set_heartbeat_interval(&self, new_heartbeat_interval: u64) {
        // Interval of how often to refill the bucket.
        const REFILL_INTERVAL: Duration = Duration::from_millis(RESET_DURATION_MILLISECONDS);

        self.heartbeat_interval
            .store(new_heartbeat_interval, Ordering::Release);

        // Number of commands allotted to the user per reset period.
        let commands_allotted = u32::from(available_commands_per_interval(new_heartbeat_interval));

        // This will attempt to set the ratelimiter to a new one based on the
        // heartbeat interval. The OnceCell may contain either Some if
        // ratelimiting is enabled, or None if it was disabled.
        // If it was already disabled or previously enabled, setting the inner
        // value will fail and therefore errors should be ignored.
        let _result = self.ratelimit.set(Some(
            LeakyBucket::builder()
                .max(commands_allotted)
                .tokens(commands_allotted)
                .refill_interval(REFILL_INTERVAL)
                .refill_amount(commands_allotted)
                .build(),
        ));
    }

    /// Returns the current sequence.
    pub fn seq(&self) -> u64 {
        self.seq.load(Ordering::Relaxed)
    }

    /// Sets the sequence.
    pub fn set_seq(&self, seq: u64) {
        self.seq.store(seq, Ordering::Release);
    }

    /// Returns the current shard stage.
    pub fn stage(&self) -> Stage {
        Stage::try_from(self.stage.load(Ordering::Relaxed)).unwrap_or_default()
    }

    /// Sets the stage.
    pub fn set_stage(&self, stage: Stage) {
        self.stage.store(stage as u8, Ordering::Release);
    }

    pub fn heartbeat(&self) -> Result<(), SessionSendError> {
        self.send(Heartbeat::new(self.seq()))
    }

    pub fn id(&self) -> Option<Box<str>> {
        self.id.lock().expect("id poisoned").clone()
    }

    pub fn set_id(&self, new_id: Box<str>) {
        self.id.lock().expect("id poisoned").replace(new_id);
    }

    pub fn stop_heartbeater(&self) {
        if let Some(handle) = self
            .heartbeater_handle
            .lock()
            .expect("heartbeater poisoned")
            .take()
        {
            handle.abort();
        }
    }

    pub fn start_heartbeater(&self) {
        let interval = self.heartbeat_interval();
        let seq = Arc::clone(&self.seq);
        let heartbeats = Arc::clone(&self.heartbeats);

        let heartbeater = Heartbeater::new(heartbeats, interval, seq, self.tx.clone()).run();
        let handle = tokio::spawn(heartbeater);

        if let Some(old) = self
            .heartbeater_handle
            .lock()
            .expect("heartbeater poisoned")
            .replace(handle)
        {
            old.abort();
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.stop_heartbeater();
    }
}

/// Calculate the number of commands to allot in a given reset period while
/// taking the heartbeat interval into account.
///
/// This is reserving twice as much as needed for heartbeats, to account for
/// Discord sending us a heartbeat and expecting a heartbeat in response.
///
/// For example, when the heartbeat interval is 42500 milliseconds then 116
/// commands will be allotted per reset period.
fn available_commands_per_interval(heartbeat_interval: u64) -> u8 {
    // Number of commands allowed in a given reset period.
    //
    // API documentation with details:
    // <https://discord.com/developers/docs/topics/gateway#rate-limiting>
    const COMMANDS_PER_RESET: u8 = 120;

    let mut heartbeats = RESET_DURATION_MILLISECONDS / heartbeat_interval;
    let remainder = RESET_DURATION_MILLISECONDS % heartbeat_interval;

    // If we have a remainder then we reserve an additional heartbeat.
    //
    // If there is a remainder per reset then in theory we could allot one less
    // command for heartbeating variably every number of resets, but it's best
    // to be cautious and keep it simple.
    if remainder > 0 {
        heartbeats += 1;
    }

    // Convert the heartbeats to a u8. The number of heartbeats **should** never
    // be above `u8::MAX`, so the error pattern branch should never be reached.
    let heartbeats_converted = if let Ok(value) = heartbeats.try_into() {
        value
    } else {
        // Number of commands to reserve per reset. This number is a bit
        // high because the heartbeat interval may be anything, so we're
        // just being cautious here.
        const ALLOT_ON_FAIL: u8 = COMMANDS_PER_RESET - 10;

        tracing::warn!(
            %heartbeats,
            "heartbeats > u8 max; defaulting to allotting {ALLOT_ON_FAIL}",
        );

        ALLOT_ON_FAIL
    };

    COMMANDS_PER_RESET.saturating_sub(heartbeats_converted * 2)
}

#[cfg(test)]
mod tests {
    #[test]
    fn heartbeats_per_reset() {
        assert_eq!(118, super::available_commands_per_interval(60_000));
        assert_eq!(116, super::available_commands_per_interval(42_500));
        assert_eq!(116, super::available_commands_per_interval(30_000));
        assert_eq!(114, super::available_commands_per_interval(29_999));
    }
}
