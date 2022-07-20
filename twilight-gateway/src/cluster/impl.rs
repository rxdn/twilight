use super::{ClusterBuilder, Config, Events};
use crate::{
    cluster::event::ShardEventsWithId,
    shard::{
        raw_message::Message, Command, Config as ShardConfig, Information, ResumeSession, Shard,
    },
    Intents,
};
use futures_util::{future, stream::SelectAll};
use std::{
    collections::{hash_map::Values, HashMap},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    iter::FusedIterator,
};

/// Sending a command to a shard failed.
#[derive(Debug)]
pub struct ClusterCommandError {
    pub(super) kind: ClusterCommandErrorType,
    pub(super) source: Option<Box<dyn Error + Send + Sync>>,
}

impl ClusterCommandError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &ClusterCommandErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        ClusterCommandErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, self.source)
    }
}

impl Display for ClusterCommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            ClusterCommandErrorType::Sending => {
                f.write_str("sending the message over the websocket failed")
            }
            ClusterCommandErrorType::ShardNonexistent { id } => {
                f.write_str("shard ")?;
                Display::fmt(id, f)?;

                f.write_str(" does not exist")
            }
        }
    }
}

impl Error for ClusterCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`ClusterCommandError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum ClusterCommandErrorType {
    /// The shard exists, but sending the provided value failed.
    Sending,
    /// Provided shard ID does not exist.
    ShardNonexistent {
        /// Provided shard ID.
        id: u64,
    },
}

/// Sending a raw websocket message via a shard failed.
#[derive(Debug)]
pub struct ClusterSendError {
    kind: ClusterSendErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl ClusterSendError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &ClusterSendErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (ClusterSendErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, self.source)
    }
}

impl Display for ClusterSendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            ClusterSendErrorType::Sending => f.write_str("failed to send message over websocket"),
            ClusterSendErrorType::ShardNonexistent { id } => {
                f.write_str("shard ")?;
                Display::fmt(id, f)?;

                f.write_str(" does not exist")
            }
        }
    }
}

impl Error for ClusterSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`ClusterSendError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum ClusterSendErrorType {
    /// The shard exists, but sending the provided value failed.
    Sending,
    /// Provided shard ID does not exist.
    ShardNonexistent {
        /// Provided shard ID.
        id: u64,
    },
}

/// Starting a cluster failed.
#[derive(Debug)]
pub struct ClusterStartError {
    pub(super) kind: ClusterStartErrorType,
    pub(super) source: Option<Box<dyn Error + Send + Sync>>,
}

impl ClusterStartError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &ClusterStartErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (ClusterStartErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, self.source)
    }
}

impl Display for ClusterStartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            ClusterStartErrorType::AutoSharding => {
                f.write_str("retrieving the bot's recommended number of shards failed")
            }
            ClusterStartErrorType::Tls => {
                f.write_str("creating the TLS connector resulted in a error")
            }
        }
    }
}

impl Error for ClusterStartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`ClusterStartErrorType`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum ClusterStartErrorType {
    /// Retrieving the bot's recommended number of shards via the HTTP API
    /// failed.
    AutoSharding,
    /// Creating the TLS connector resulted in a error.
    Tls,
}

/// A manager for multiple shards.
///
/// # Using a cluster in multiple tasks
///
/// To use a cluster instance in multiple tasks, consider wrapping it in an
/// [`std::sync::Arc`] or [`std::rc::Rc`].
///
/// # Examples
///
/// Refer to the module-level documentation for examples.
#[derive(Debug)]
pub struct Cluster {
    config: Config,
    shards: HashMap<u64, Shard>,
}

impl Cluster {
    /// Create a new unconfigured cluster.
    ///
    /// Use [`builder`] to configure and construct a cluster.
    ///
    /// # Examples
    ///
    /// Create a cluster, receiving a stream of events:
    ///
    /// ```no_run
    /// use twilight_gateway::{Cluster, EventTypeFlags, Event, Intents};
    /// use futures::StreamExt;
    /// use std::env;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let types = EventTypeFlags::MESSAGE_CREATE
    ///     | EventTypeFlags::MESSAGE_DELETE
    ///     | EventTypeFlags::MESSAGE_UPDATE;
    ///
    /// let (cluster, mut events) = Cluster::builder(env::var("DISCORD_TOKEN")?, Intents::GUILD_MESSAGES)
    ///     .event_types(types)
    ///     .build()
    ///     .await?;
    /// cluster.up().await;
    ///
    /// while let Some((shard_id, event)) = events.next().await {
    ///     match event {
    ///         Event::MessageCreate(_) => println!("Shard {shard_id} got a new message"),
    ///         Event::MessageDelete(_) => println!("Shard {shard_id} got a deleted message"),
    ///         Event::MessageUpdate(_) => println!("Shard {shard_id} got an updated message"),
    ///         // No other events will come in through the stream.
    ///         _ => {},
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClusterStartErrorType::AutoSharding`] error type if
    /// there was an HTTP error retrieving the number of recommended shards.
    ///
    /// [`builder`]: Self::builder
    pub async fn new(token: String, intents: Intents) -> Result<(Self, Events), ClusterStartError> {
        Self::builder(token, intents).build().await
    }

    pub(super) fn new_with_config(
        mut config: Config,
        shard_config: &ShardConfig,
    ) -> (Self, Events) {
        #[derive(Default)]
        struct ShardFold {
            shards: HashMap<u64, Shard>,
            streams: Vec<ShardEventsWithId>,
        }

        let total = config.shard_scheme().total();

        #[cfg(feature = "metrics")]
        #[allow(clippy::cast_precision_loss)]
        {
            metrics::gauge!("Cluster-Shard-Count", total as f64);
        }

        let ShardFold { shards, streams } =
            config
                .shard_scheme()
                .iter()
                .fold(ShardFold::default(), |mut fold, idx| {
                    let mut shard_config = shard_config.clone();
                    shard_config.shard = [idx, total];

                    if let Some(data) = config.resume_sessions.remove(&idx) {
                        shard_config.session_id = Some(data.session_id.into_boxed_str());
                        shard_config.sequence = Some(data.sequence);
                    }

                    if let Some(shard_presence) = &config.shard_presence {
                        shard_config.presence = shard_presence(idx)
                    }

                    let (shard, stream) = Shard::new_with_config(shard_config);

                    fold.shards.insert(idx, shard);
                    fold.streams.push(ShardEventsWithId::new(idx, stream));

                    fold
                });

        #[allow(clippy::from_iter_instead_of_collect)]
        let select_all = SelectAll::from_iter(streams);

        (Self { config, shards }, Events::new(select_all))
    }

    /// Create a builder to configure and construct a cluster.
    ///
    /// # Examples
    ///
    /// Create a cluster, receiving a stream of events when a message is
    /// created, deleted, or updated:
    ///
    /// ```no_run
    /// use twilight_gateway::{Cluster, EventTypeFlags, Event, Intents};
    /// use futures::StreamExt;
    /// use std::env;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let types = EventTypeFlags::MESSAGE_CREATE
    ///     | EventTypeFlags::MESSAGE_DELETE
    ///     | EventTypeFlags::MESSAGE_UPDATE;
    ///
    /// let (cluster, mut events) = Cluster::builder(token, Intents::GUILD_MESSAGES)
    ///     .event_types(types)
    ///     .build()
    ///     .await?;
    /// cluster.up().await;
    ///
    /// while let Some((shard_id, event)) = events.next().await {
    ///     match event {
    ///         Event::MessageCreate(_) => println!("Shard {shard_id} got a new message"),
    ///         Event::MessageDelete(_) => println!("Shard {shard_id} got a deleted message"),
    ///         Event::MessageUpdate(_) => println!("Shard {shard_id} got an updated message"),
    ///         // No other events will come in through the stream.
    ///         _ => {},
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
    pub fn builder(token: String, intents: Intents) -> ClusterBuilder {
        ClusterBuilder::new(token, intents)
    }

    /// Return an immutable reference to the configuration of this cluster.
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Bring up the cluster, starting all of the shards that it was configured
    /// to manage.
    ///
    /// # Examples
    ///
    /// Bring up a cluster, starting shards all 10 shards that a bot uses:
    ///
    /// ```no_run
    /// use twilight_gateway::{cluster::{Cluster, ShardScheme}, Intents};
    /// use std::env;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let scheme = ShardScheme::try_from((0..=9, 10))?;
    /// let (cluster, _) = Cluster::builder(token, Intents::GUILD_MESSAGES)
    ///     .shard_scheme(scheme)
    ///     .build()
    ///     .await?;
    ///
    /// // Finally, bring up the cluster.
    /// cluster.up().await;
    /// # Ok(()) }
    /// ```
    pub async fn up(&self) {
        future::join_all(self.shards.values().map(Shard::start)).await;
    }

    /// Bring down the cluster, stopping all of the shards that it's managing.
    pub fn down(&self) {
        for shard in self.shards.values() {
            shard.shutdown();
        }
    }

    /// Bring down the cluster in a resumable way and returns all info needed
    /// for resuming.
    ///
    /// The returned map is keyed by the shard's ID to the information needed
    /// to resume. If a shard can't resume, then it is not included in the map.
    ///
    /// **Note**: Discord only allows resuming for a few minutes after
    /// disconnection. You may also not be able to resume if you missed too many
    /// events already.
    pub fn down_resumable(&self) -> HashMap<u64, ResumeSession> {
        self.shards
            .values()
            .map(Shard::shutdown_resumable)
            .filter_map(|(id, session)| session.map(|s| (id, s)))
            .collect()
    }

    /// Return a Shard by its ID.
    pub fn shard(&self, id: u64) -> Option<&Shard> {
        self.shards.get(&id)
    }

    /// Return an iterator of all the shards.
    pub fn shards(&self) -> Shards<'_> {
        Shards {
            iter: self.shards.values(),
        }
    }

    /// Return information about all shards.
    ///
    /// # Examples
    ///
    /// After waiting a minute, print the ID, latency, and stage of each shard:
    ///
    /// ```no_run
    /// use twilight_gateway::{Cluster, Intents};
    /// use std::{env, time::Duration};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (cluster, _) = Cluster::new(env::var("DISCORD_TOKEN")?, Intents::empty()).await?;
    /// cluster.up().await;
    ///
    /// tokio::time::sleep(Duration::from_secs(60)).await;
    ///
    /// for (shard_id, info) in cluster.info() {
    ///     println!(
    ///         "Shard {shard_id} is {} with an average latency of {:?}",
    ///         info.stage(),
    ///         info.latency().average(),
    ///     );
    /// }
    /// # Ok(()) }
    /// ```
    pub fn info(&self) -> HashMap<u64, Information> {
        self.shards
            .iter()
            .filter_map(|(id, shard)| shard.info().ok().map(|info| (*id, info)))
            .collect()
    }

    /// Send a command to the specified shard.
    ///
    /// # Examples
    ///
    /// Update the current user's presence on shard ID 2:
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::env;
    /// use twilight_gateway::{cluster::Cluster, Intents};
    /// use twilight_model::{
    ///     gateway::{
    ///         payload::outgoing::UpdatePresence,
    ///         presence::{Activity, ActivityType, MinimalActivity, Status},
    ///     },
    /// };
    ///
    /// let intents = Intents::GUILD_VOICE_STATES;
    /// let token = env::var("DISCORD_TOKEN")?;
    ///
    /// let (cluster, _events) = Cluster::new(token, intents).await?;
    ///
    /// // Wait for shards to come up before sending a message to one of them.
    /// cluster.up().await;
    ///
    /// // Update the user's presence to a custom activity with a name of
    /// // "testing".
    /// let activity = Activity::from(MinimalActivity {
    ///     kind: ActivityType::Custom,
    ///     name: "testing".to_owned(),
    ///     url: None,
    /// });
    /// let request = UpdatePresence::new(
    ///     Vec::from([activity]),
    ///     false,
    ///     None,
    ///     Status::Online,
    /// )?;
    ///
    /// // Send the request over the shard.
    /// cluster.command(2, &request).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClusterCommandErrorType::Sending`] error type if the shard
    /// exists, but sending it failed.
    ///
    /// Returns a [`ClusterCommandErrorType::ShardNonexistent`] error type if
    /// the provided shard ID does not exist in the cluster.
    pub async fn command(&self, id: u64, value: &impl Command) -> Result<(), ClusterCommandError> {
        let shard = self.shard(id).ok_or(ClusterCommandError {
            kind: ClusterCommandErrorType::ShardNonexistent { id },
            source: None,
        })?;

        shard
            .command(value)
            .await
            .map_err(|source| ClusterCommandError {
                kind: ClusterCommandErrorType::Sending,
                source: Some(Box::new(source)),
            })
    }

    /// Send a raw websocket message.
    ///
    /// # Examples
    ///
    /// Send a restart close to shard ID 7:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::env;
    /// use twilight_gateway::{
    ///     cluster::Cluster,
    ///     shard::raw_message::{CloseFrame, Message},
    ///     Intents,
    /// };
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let (cluster, _) = Cluster::new(token, Intents::GUILDS).await?;
    /// cluster.up().await;
    ///
    /// // some time later..
    /// let close = CloseFrame::from((1012, ""));
    /// let message = Message::Close(Some(close));
    /// cluster.send(7, message).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`ClusterCommandErrorType::Sending`] error type if the shard
    /// exists, but sending the close code failed.
    ///
    /// Returns a [`ClusterCommandErrorType::ShardNonexistent`] error type if
    /// the provided shard ID does not exist in the cluster.
    pub async fn send(&self, id: u64, message: Message) -> Result<(), ClusterSendError> {
        let shard = self.shard(id).ok_or(ClusterSendError {
            kind: ClusterSendErrorType::ShardNonexistent { id },
            source: None,
        })?;

        shard
            .send(message)
            .await
            .map_err(|source| ClusterSendError {
                kind: ClusterSendErrorType::Sending,
                source: Some(Box::new(source)),
            })
    }
}

/// Iterator over a [`Cluster`]'s managed [shards][`Shard`].
///
/// This is returned by [`Cluster::shards`].
pub struct Shards<'a> {
    iter: Values<'a, u64, Shard>,
}

impl ExactSizeIterator for Shards<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl FusedIterator for Shards<'_> {}

impl<'a> Iterator for Shards<'a> {
    type Item = &'a Shard;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Cluster, ClusterCommandError, ClusterCommandErrorType, ClusterSendError,
        ClusterSendErrorType, ClusterStartError, ClusterStartErrorType,
    };
    use static_assertions::{assert_fields, assert_impl_all};
    use std::{error::Error, fmt::Debug};

    assert_impl_all!(ClusterCommandErrorType: Debug, Send, Sync);
    assert_fields!(ClusterCommandErrorType::ShardNonexistent: id);
    assert_impl_all!(ClusterCommandError: Error, Send, Sync);
    assert_impl_all!(ClusterSendErrorType: Debug, Send, Sync);
    assert_fields!(ClusterSendErrorType::ShardNonexistent: id);
    assert_impl_all!(ClusterSendError: Error, Send, Sync);
    assert_impl_all!(ClusterStartErrorType: Debug, Send, Sync);
    assert_impl_all!(ClusterStartError: Error, Send, Sync);
    assert_impl_all!(Cluster: Debug, Send, Sync);
}
