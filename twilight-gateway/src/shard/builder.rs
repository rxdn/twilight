use super::{Config, Events, Shard};
use crate::EventTypeFlags;
use std::{
    borrow::Cow,
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    sync::Arc,
};
use twilight_gateway_queue::{LocalQueue, Queue};
use twilight_model::gateway::{
    payload::outgoing::{identify::IdentifyProperties, update_presence::UpdatePresencePayload},
    Intents,
};

/// Shard ID configuration is invalid.
///
/// Returned by [`ShardBuilder::shard`].
#[derive(Debug)]
pub struct ShardIdError {
    kind: ShardIdErrorType,
}

impl ShardIdError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &ShardIdErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (ShardIdErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for ShardIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            ShardIdErrorType::IdTooLarge { id, total } => {
                f.write_str("provided shard ID ")?;
                Display::fmt(id, f)?;
                f.write_str(" is larger than the total ")?;

                Display::fmt(total, f)
            }
        }
    }
}

impl Error for ShardIdError {}

/// Type of [`ShardIdError`] that occurred.
#[derive(Debug)]
pub enum ShardIdErrorType {
    /// Provided shard ID is higher than provided total shard count.
    IdTooLarge {
        /// Shard ID.
        id: u64,
        /// Total shard count.
        total: u64,
    },
}

/// Builder to configure and construct a shard.
///
/// Use [`ShardBuilder::new`] to start configuring a new [`Shard`].
///
/// # Examples
///
/// Create a new shard, setting the [`large_threshold`] to 100 and the
/// [`shard`] ID to 5 out of 10:
///
/// ```no_run
/// use std::env;
/// use twilight_gateway::{Intents, Shard};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let token = env::var("DISCORD_TOKEN")?;
///
/// let shard = Shard::builder(token, Intents::GUILD_MESSAGE_REACTIONS)
///     .large_threshold(100)
///     .shard(5, 10)?
///     .build();
/// # Ok(()) }
/// ```
///
/// [`ShardBuilder::new`]: Self::new
/// [`large_threshold`]: Self::large_threshold
/// [`shard`]: Self::shard
#[derive(Debug)]
#[must_use = "has no effect if not built"]
pub struct ShardBuilder {
    event_types: EventTypeFlags,
    pub(crate) gateway_url: Option<String>,
    identify_properties: Option<IdentifyProperties>,
    intents: Intents,
    large_threshold: u64,
    presence: Option<UpdatePresencePayload>,
    queue: Arc<dyn Queue>,
    ratelimit_payloads: bool,
    shard: [u64; 2],
    token: Box<str>,
}

impl ShardBuilder {
    /// Create a new builder to configure and construct a shard.
    ///
    /// Refer to each method to learn their default values.
    pub fn new(mut token: String, intents: Intents) -> Self {
        if !token.starts_with("Bot ") {
            token.insert_str(0, "Bot ");
        }

        Self {
            event_types: EventTypeFlags::default(),
            gateway_url: None,
            identify_properties: None,
            intents,
            large_threshold: 50,
            presence: None,
            queue: Arc::new(LocalQueue::new()),
            ratelimit_payloads: true,
            shard: [0, 1],
            token: token.into_boxed_str(),
        }
    }

    pub(crate) fn into_config(self) -> Config {
        Config {
            event_types: self.event_types,
            gateway_url: match self.gateway_url {
                Some(s) => Cow::Owned(s),
                None => Cow::Borrowed(crate::URL),
            },
            identify_properties: self.identify_properties,
            intents: self.intents,
            large_threshold: self.large_threshold,
            presence: self.presence,
            queue: self.queue,
            ratelimit_payloads: self.ratelimit_payloads,
            session_id: None,
            sequence: None,
            shard: self.shard,
            #[cfg(any(
                feature = "native",
                feature = "rustls-native-roots",
                feature = "rustls-webpki-roots"
            ))]
            tls: None,
            token: self.token,
        }
    }

    /// Consume the builder, constructing a shard.
    pub fn build(self) -> (Shard, Events) {
        Shard::new_with_config(self.into_config())
    }

    /// Set the event types to process.
    ///
    /// This is an optimization technique; all events not included in the
    /// provided event type flags will not be deserialized by the gateway and
    /// will be discarded. All events will still be sent if
    /// [`EventTypeFlags::SHARD_PAYLOAD`] is enabled.
    ///
    /// [`EventTypeFlags::SHARD_PAYLOAD`]: crate::EventTypeFlags::SHARD_PAYLOAD
    pub const fn event_types(mut self, event_types: EventTypeFlags) -> Self {
        self.event_types = event_types;

        self
    }

    /// Set the proxy URL for connecting to the gateway.
    ///
    /// Default is to use Discord's gateway URL.
    #[allow(clippy::missing_const_for_fn)]
    pub fn gateway_url(mut self, gateway_url: String) -> Self {
        self.gateway_url = Some(gateway_url);

        self
    }

    /// Set the properties to identify with.
    ///
    /// This may be used if you want to set a different operating system, for
    /// example.
    ///
    /// # Examples
    ///
    /// Set the identify properties for a shard:
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::env::{self, consts::OS};
    /// use twilight_gateway::{Intents, Shard};
    /// use twilight_model::gateway::payload::outgoing::identify::IdentifyProperties;
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let properties = IdentifyProperties::new("twilight.rs", "twilight.rs", OS);
    ///
    /// let builder = Shard::builder(token, Intents::empty())
    ///     .identify_properties(properties);
    /// # Ok(()) }
    /// ```
    #[allow(clippy::missing_const_for_fn)]
    pub fn identify_properties(mut self, identify_properties: IdentifyProperties) -> Self {
        self.identify_properties = Some(identify_properties);

        self
    }

    /// Set the maximum number of members in a guild to load the member list.
    ///
    /// Default value is `50`. The minimum value is `50` and the maximum is
    /// `250`.
    ///
    /// # Examples
    ///
    /// If you pass `200`, then if there are 250 members in a guild the member
    /// list won't be sent. If there are 150 members, then the list *will* be
    /// sent.
    ///
    /// # Panics
    ///
    /// Panics if the provided value is below 50 or above 250.
    #[track_caller]
    pub fn large_threshold(mut self, large_threshold: u64) -> Self {
        match large_threshold {
            0..=49 => panic!("threshold {large_threshold} is below 50"),
            50..=250 => (),
            251.. => panic!("threshold {large_threshold} is above 250"),
        }

        self.large_threshold = large_threshold;

        self
    }

    /// Set the presence to use automatically when starting a new session.
    ///
    /// Default is no presence, which defaults to strictly being "online"
    /// with no special qualities.
    ///
    /// # Examples
    ///
    /// Set the bot user's presence to idle with the status "Not accepting
    /// commands":
    ///
    /// ```no_run
    /// use std::env;
    /// use twilight_gateway::{Intents, Shard};
    /// use twilight_model::gateway::{
    ///     payload::outgoing::update_presence::UpdatePresencePayload,
    ///     presence::{ActivityType, MinimalActivity, Status},
    /// };
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let shard = Shard::builder(env::var("DISCORD_TOKEN")?, Intents::empty())
    ///     .presence(UpdatePresencePayload::new(
    ///         vec![MinimalActivity {
    ///             kind: ActivityType::Playing,
    ///             name: "Not accepting commands".into(),
    ///             url: None,
    ///         }
    ///         .into()],
    ///         false,
    ///         None,
    ///         Status::Idle,
    ///     )?);
    /// # Ok(()) }
    ///
    /// ```
    pub fn presence(mut self, presence: UpdatePresencePayload) -> Self {
        self.presence.replace(presence);

        self
    }

    /// Set the queue to use for queueing shard connections.
    ///
    /// You probably don't need to set this yourself, because the [`Cluster`]
    /// manages that for you. Refer to the [`queue`] module for more
    /// information.
    ///
    /// The default value is a queue used only by this shard, or a queue used by
    /// all shards when ran by a [`Cluster`].
    ///
    /// [`Cluster`]: crate::cluster::Cluster
    /// [`queue`]: crate::queue
    pub fn queue(mut self, queue: Arc<dyn Queue>) -> Self {
        self.queue = queue;

        self
    }

    /// Set whether or not outgoing payloads will be ratelimited.
    ///
    /// Useful when running behind a proxy gateway. Running without a
    /// functional ratelimiter **will** get you ratelimited.
    ///
    /// Defaults to being enabled.
    #[allow(clippy::missing_const_for_fn)]
    pub fn ratelimit_payloads(mut self, ratelimit_payloads: bool) -> Self {
        self.ratelimit_payloads = ratelimit_payloads;

        self
    }

    /// Set the shard ID to connect as, and the total number of shards used by
    /// the bot.
    ///
    /// The shard ID is 0-indexed, while the total is 1-indexed.
    ///
    /// The default value is a shard ID of 0 and a shard total of 1, which is
    /// good for smaller bots.
    ///
    /// **Note**: If your bot is in over 250'000 guilds then `shard_total`
    /// *should probably* be a multiple of 16 if you're in the "Large Bot
    /// Sharding" program.
    ///
    /// # Examples
    ///
    /// If you have 19 shards, then your last shard will have an ID of 18 out of
    /// a total of 19 shards:
    ///
    /// ```no_run
    /// use twilight_gateway::{Intents, Shard};
    /// use std::env;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let token = env::var("DISCORD_TOKEN")?;
    ///
    /// let shard = Shard::builder(token, Intents::empty()).shard(18, 19)?.build();
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ShardIdErrorType::IdTooLarge`] error type if the shard ID to
    /// connect as is larger than the total.
    #[allow(clippy::missing_const_for_fn)]
    pub fn shard(mut self, shard_id: u64, shard_total: u64) -> Result<Self, ShardIdError> {
        if shard_id >= shard_total {
            return Err(ShardIdError {
                kind: ShardIdErrorType::IdTooLarge {
                    id: shard_id,
                    total: shard_total,
                },
            });
        }

        self.shard = [shard_id, shard_total];

        Ok(self)
    }
}

impl From<(String, Intents)> for ShardBuilder {
    fn from((token, intents): (String, Intents)) -> Self {
        Self::new(token, intents)
    }
}

#[cfg(test)]
mod tests {
    use super::{ShardBuilder, ShardIdError, ShardIdErrorType};
    use crate::Intents;
    use static_assertions::{assert_fields, assert_impl_all};
    use std::{error::Error, fmt::Debug};

    assert_impl_all!(ShardBuilder: Debug, From<(String, Intents)>, Send, Sync);
    assert_impl_all!(ShardIdErrorType: Debug, Send, Sync);
    assert_fields!(ShardIdErrorType::IdTooLarge: id, total);
    assert_impl_all!(ShardIdError: Error, Send, Sync);
}
