#![deny(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::pedantic,
    future_incompatible,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_code,
    unused
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod iter;
pub mod model;

#[cfg(feature = "permission-calculator")]
pub mod permission;

mod builder;
mod config;
mod event;
mod stats;

#[cfg(test)]
mod test;

pub use self::{
    builder::InMemoryCacheBuilder,
    config::{Config, ResourceType},
    stats::InMemoryCacheStats,
};

#[cfg(feature = "permission-calculator")]
pub use self::permission::InMemoryCachePermissions;

use self::{
    iter::InMemoryCacheIter,
    model::{
        CachedEmoji, CachedGuild, CachedMember, CachedMessage, CachedPresence, CachedSticker,
        CachedVoiceState,
    },
};
use dashmap::{
    mapref::{entry::Entry, one::Ref},
    DashMap, DashSet,
};
use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::Hash,
    ops::Deref,
    sync::Mutex,
};
use twilight_model::{
    channel::{Channel, StageInstance},
    gateway::event::Event,
    guild::{GuildIntegration, Role},
    id::{
        marker::{
            ChannelMarker, EmojiMarker, GuildMarker, IntegrationMarker, MessageMarker, RoleMarker,
            StageMarker, StickerMarker, UserMarker,
        },
        Id,
    },
    user::{CurrentUser, User},
};

/// Resource associated with a guild.
///
/// This is used when a resource does not itself include its associated guild's
/// ID. In lieu of the resource itself storing its guild's ID this relation
/// includes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildResource<T> {
    guild_id: Id<GuildMarker>,
    value: T,
}

impl<T> GuildResource<T> {
    /// ID of the guild associated with the resource.
    pub const fn guild_id(&self) -> Id<GuildMarker> {
        self.guild_id
    }

    /// Immutable reference to the resource's value.
    pub const fn resource(&self) -> &T {
        &self.value
    }
}

impl<T> Deref for GuildResource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource()
    }
}

/// Immutable reference to a resource in the cache.
// We need this so as not to expose the underlying cache implementation.
pub struct Reference<'a, K, V> {
    inner: Ref<'a, K, V>,
}

impl<'a, K: Eq + Hash, V> Reference<'a, K, V> {
    /// Create a new reference from a `DashMap` reference.
    #[allow(clippy::missing_const_for_fn)]
    fn new(inner: Ref<'a, K, V>) -> Self {
        Self { inner }
    }

    /// Immutable reference to the key identifying the resource.
    pub fn key(&'a self) -> &'a K {
        self.inner.key()
    }

    /// Immutable reference to the underlying value.
    pub fn value(&'a self) -> &'a V {
        self.inner.value()
    }
}

impl<K: Eq + Hash, V: Debug> Debug for Reference<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Reference")
            .field("inner", self.value())
            .finish()
    }
}

impl<'a, K: Eq + Hash, V> Deref for Reference<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

fn upsert_guild_item<K: Eq + Hash, V: PartialEq>(
    map: &DashMap<K, GuildResource<V>>,
    guild_id: Id<GuildMarker>,
    key: K,
    value: V,
) {
    match map.entry(key) {
        Entry::Occupied(entry) if entry.get().value == value => {}
        Entry::Occupied(mut entry) => {
            entry.insert(GuildResource { guild_id, value });
        }
        Entry::Vacant(entry) => {
            entry.insert(GuildResource { guild_id, value });
        }
    }
}

/// An in-memory cache of Discord data.
///
/// This is an implementation of a cache designed to be used by only the
/// current process.
///
/// Events will only be processed if they are properly expressed with
/// [`Intents`]; refer to function-level documentation for more details.
///
/// # Using the cache in multiple tasks
///
/// To use a cache instance in multiple tasks, consider wrapping it in an
/// [`std::sync::Arc`] or [`std::rc::Rc`].
///
/// # Caution required
///
/// The cache uses a concurrent map for mutability of cached resources. Return
/// types of methods are immutable references to those resources. If a resource
/// is retrieved from the cache then care must be taken to only hold it for as long as
/// necessary. If the cache needs to mutate a resource to update it and a
/// reference to it is being held then calls to [`InMemoryCache::update`] may
/// be blocked.
///
/// In order to avoid blocking of cache updates care must be taken to hold them
/// for as little as possible. For example, consider dropping references during
/// long-running tasks such as HTTP requests. Processing HTTP requests takes
/// milliseconds to seconds; retrieving a new reference to a resource is on the
/// scale of nanoseconds. If only a couple of small fields are necessary from a
/// reference consider copying or cloning them.
///
/// [`Intents`]: ::twilight_model::gateway::Intents
// When adding a field here, be sure to add it to `InMemoryCache::clear` if
// necessary.
#[derive(Debug, Default)]
pub struct InMemoryCache {
    config: Config,
    channels: DashMap<Id<ChannelMarker>, Channel>,
    channel_messages: DashMap<Id<ChannelMarker>, VecDeque<Id<MessageMarker>>>,
    // So long as the lock isn't held across await or panic points this is fine.
    current_user: Mutex<Option<CurrentUser>>,
    emojis: DashMap<Id<EmojiMarker>, GuildResource<CachedEmoji>>,
    guilds: DashMap<Id<GuildMarker>, CachedGuild>,
    guild_channels: DashMap<Id<GuildMarker>, HashSet<Id<ChannelMarker>>>,
    guild_emojis: DashMap<Id<GuildMarker>, HashSet<Id<EmojiMarker>>>,
    guild_integrations: DashMap<Id<GuildMarker>, HashSet<Id<IntegrationMarker>>>,
    guild_members: DashMap<Id<GuildMarker>, HashSet<Id<UserMarker>>>,
    guild_presences: DashMap<Id<GuildMarker>, HashSet<Id<UserMarker>>>,
    guild_roles: DashMap<Id<GuildMarker>, HashSet<Id<RoleMarker>>>,
    guild_stage_instances: DashMap<Id<GuildMarker>, HashSet<Id<StageMarker>>>,
    guild_stickers: DashMap<Id<GuildMarker>, HashSet<Id<StickerMarker>>>,
    integrations:
        DashMap<(Id<GuildMarker>, Id<IntegrationMarker>), GuildResource<GuildIntegration>>,
    members: DashMap<(Id<GuildMarker>, Id<UserMarker>), CachedMember>,
    messages: DashMap<Id<MessageMarker>, CachedMessage>,
    presences: DashMap<(Id<GuildMarker>, Id<UserMarker>), CachedPresence>,
    roles: DashMap<Id<RoleMarker>, GuildResource<Role>>,
    stage_instances: DashMap<Id<StageMarker>, GuildResource<StageInstance>>,
    stickers: DashMap<Id<StickerMarker>, GuildResource<CachedSticker>>,
    unavailable_guilds: DashSet<Id<GuildMarker>>,
    users: DashMap<Id<UserMarker>, User>,
    user_guilds: DashMap<Id<UserMarker>, BTreeSet<Id<GuildMarker>>>,
    /// Mapping of channels and the users currently connected.
    #[allow(clippy::type_complexity)]
    voice_state_channels: DashMap<Id<ChannelMarker>, HashSet<(Id<GuildMarker>, Id<UserMarker>)>>,
    /// Mapping of guilds and users currently connected to its voice channels.
    voice_state_guilds: DashMap<Id<GuildMarker>, HashSet<Id<UserMarker>>>,
    /// Mapping of guild ID and user ID pairs to their voice states.
    voice_states: DashMap<(Id<GuildMarker>, Id<UserMarker>), CachedVoiceState>,
}

/// Implemented methods and types for the cache.
impl InMemoryCache {
    /// Creates a new, empty cache.
    ///
    /// # Examples
    ///
    /// Creating a new `InMemoryCache` with a custom configuration, limiting
    /// the message cache to 50 messages per channel:
    ///
    /// ```
    /// use twilight_cache_inmemory::InMemoryCache;
    ///
    /// let cache = InMemoryCache::builder().message_cache_size(50).build();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new builder to configure and construct an in-memory cache.
    pub const fn builder() -> InMemoryCacheBuilder {
        InMemoryCacheBuilder::new()
    }

    /// Clear the state of the Cache.
    ///
    /// This is equal to creating a new empty cache.
    pub fn clear(&self) {
        self.channels.clear();
        self.channel_messages.clear();
        self.current_user
            .lock()
            .expect("current user poisoned")
            .take();
        self.emojis.clear();
        self.guilds.clear();
        self.guild_channels.clear();
        self.guild_emojis.clear();
        self.guild_integrations.clear();
        self.guild_members.clear();
        self.guild_presences.clear();
        self.guild_roles.clear();
        self.guild_stage_instances.clear();
        self.guild_stickers.clear();
        self.integrations.clear();
        self.members.clear();
        self.messages.clear();
        self.presences.clear();
        self.roles.clear();
        self.stickers.clear();
        self.unavailable_guilds.clear();
        self.users.clear();
        self.voice_state_channels.clear();
        self.voice_state_guilds.clear();
        self.voice_states.clear();
    }

    /// Returns a copy of the config cache.
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Create an interface for iterating over the various resources in the
    /// cache.
    ///
    /// Via the iterator interface many resource types can be iterated over
    /// including, but not limited to, emojis, guilds, presences, and users.
    ///
    /// # Examples
    ///
    /// Iterate over every guild in the cache and print their IDs and names:
    ///
    /// ```no_run
    /// use twilight_cache_inmemory::InMemoryCache;
    ///
    /// let cache = InMemoryCache::new();
    ///
    /// // later in the application...
    /// for guild in cache.iter().guilds() {
    ///     println!("{}: {}", guild.id(), guild.name());
    /// }
    /// ```
    #[allow(clippy::iter_not_returning_iterator)]
    pub const fn iter(&self) -> InMemoryCacheIter<'_> {
        InMemoryCacheIter::new(self)
    }

    /// Create an interface for retrieving statistics about the cache.
    ///
    /// # Examples
    ///
    /// Print the number of guilds in a cache:
    ///
    /// ```
    /// use twilight_cache_inmemory::InMemoryCache;
    ///
    /// let cache = InMemoryCache::new();
    ///
    /// // later on...
    /// let guilds = cache.stats().guilds();
    /// println!("guild count: {guilds}");
    /// ```
    pub const fn stats(&self) -> InMemoryCacheStats<'_> {
        InMemoryCacheStats::new(self)
    }

    /// Create an interface for retrieving the permissions of a member in a
    /// guild or channel.
    ///
    /// [`ResourceType`]s must be configured for the permission interface to
    /// properly work; refer to the [`permission`] module-level documentation
    /// for more information.
    ///
    /// # Examples
    ///
    /// Calculate the permissions of a member in a guild channel:
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use twilight_cache_inmemory::{InMemoryCache, ResourceType};
    /// use twilight_model::id::Id;
    ///
    /// let resource_types = ResourceType::CHANNEL
    ///     | ResourceType::MEMBER
    ///     | ResourceType::ROLE;
    ///
    /// let cache = InMemoryCache::builder()
    ///     .resource_types(resource_types)
    ///     .build();
    ///
    /// let channel_id = Id::new(4);
    /// let user_id = Id::new(5);
    ///
    /// let permissions = cache.permissions().in_channel(user_id, channel_id)?;
    /// println!("member has these permissions: {permissions:?}");
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "permission-calculator")]
    pub const fn permissions(&self) -> InMemoryCachePermissions<'_> {
        InMemoryCachePermissions::new(self)
    }

    /// Update the cache with an event from the gateway.
    pub fn update(&self, value: &impl UpdateCache) {
        value.update(self);
    }

    /// Gets the current user.
    pub fn current_user(&self) -> Option<CurrentUser> {
        self.current_user
            .lock()
            .expect("current user poisoned")
            .clone()
    }

    /// Gets a channel by ID.
    pub fn channel(
        &self,
        channel_id: Id<ChannelMarker>,
    ) -> Option<Reference<'_, Id<ChannelMarker>, Channel>> {
        self.channels.get(&channel_id).map(Reference::new)
    }

    /// Gets the set of messages in a channel.
    ///
    /// This requires the [`DIRECT_MESSAGES`] or [`GUILD_MESSAGES`] intents.
    ///
    /// [`DIRECT_MESSAGES`]: ::twilight_model::gateway::Intents::DIRECT_MESSAGES
    /// [`GUILD_MESSAGES`]: ::twilight_model::gateway::Intents::GUILD_MESSAGES
    pub fn channel_messages(
        &self,
        channel_id: Id<ChannelMarker>,
    ) -> Option<Reference<'_, Id<ChannelMarker>, VecDeque<Id<MessageMarker>>>> {
        self.channel_messages.get(&channel_id).map(Reference::new)
    }

    /// Gets an emoji by ID.
    ///
    /// This requires the [`GUILD_EMOJIS_AND_STICKERS`] intent.
    ///
    /// [`GUILD_EMOJIS_AND_STICKERS`]: ::twilight_model::gateway::Intents::GUILD_EMOJIS_AND_STICKERS
    pub fn emoji(
        &self,
        emoji_id: Id<EmojiMarker>,
    ) -> Option<Reference<'_, Id<EmojiMarker>, GuildResource<CachedEmoji>>> {
        self.emojis.get(&emoji_id).map(Reference::new)
    }

    /// Gets a guild by ID.
    ///
    /// This requires the [`GUILDS`] intent.
    ///
    /// [`GUILDS`]: ::twilight_model::gateway::Intents::GUILDS
    pub fn guild(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, CachedGuild>> {
        self.guilds.get(&guild_id).map(Reference::new)
    }

    /// Gets the set of channels in a guild.
    ///
    /// This requires the [`GUILDS`] intent.
    ///
    /// [`GUILDS`]: ::twilight_model::gateway::Intents::GUILDS
    pub fn guild_channels(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<ChannelMarker>>>> {
        self.guild_channels.get(&guild_id).map(Reference::new)
    }

    /// Gets the set of emojis in a guild.
    ///
    /// This requires both the [`GUILDS`] and [`GUILD_EMOJIS_AND_STICKERS`]
    /// intents.
    ///
    /// [`GUILDS`]: ::twilight_model::gateway::Intents::GUILDS
    /// [`GUILD_EMOJIS_AND_STICKERS`]: ::twilight_model::gateway::Intents::GUILD_EMOJIS_AND_STICKERS
    pub fn guild_emojis(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<EmojiMarker>>>> {
        self.guild_emojis.get(&guild_id).map(Reference::new)
    }

    /// Gets the set of integrations in a guild.
    ///
    /// This requires the [`GUILD_INTEGRATIONS`] intent. The
    /// [`ResourceType::INTEGRATION`] resource type must be enabled.
    ///
    /// [`GUILD_INTEGRATIONS`]: twilight_model::gateway::Intents::GUILD_INTEGRATIONS
    pub fn guild_integrations(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<IntegrationMarker>>>> {
        self.guild_integrations.get(&guild_id).map(Reference::new)
    }

    /// Gets the set of members in a guild.
    ///
    /// This list may be incomplete if not all members have been cached.
    ///
    /// This requires the [`GUILD_MEMBERS`] intent.
    ///
    /// [`GUILD_MEMBERS`]: ::twilight_model::gateway::Intents::GUILD_MEMBERS
    pub fn guild_members(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<UserMarker>>>> {
        self.guild_members.get(&guild_id).map(Reference::new)
    }

    /// Gets the set of presences in a guild.
    ///
    /// This list may be incomplete if not all members have been cached.
    ///
    /// This requires the [`GUILD_PRESENCES`] intent.
    ///
    /// [`GUILD_PRESENCES`]: ::twilight_model::gateway::Intents::GUILD_PRESENCES
    pub fn guild_presences(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<UserMarker>>>> {
        self.guild_presences.get(&guild_id).map(Reference::new)
    }

    /// Gets the set of roles in a guild.
    ///
    /// This requires the [`GUILDS`] intent.
    ///
    /// [`GUILDS`]: ::twilight_model::gateway::Intents::GUILDS
    pub fn guild_roles(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<RoleMarker>>>> {
        self.guild_roles.get(&guild_id).map(Reference::new)
    }

    /// Gets the set of stage instances in a guild.
    ///
    /// This requires the [`GUILDS`] intent.
    ///
    /// [`GUILDS`]: twilight_model::gateway::Intents::GUILDS
    pub fn guild_stage_instances(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<StageMarker>>>> {
        self.guild_stage_instances
            .get(&guild_id)
            .map(Reference::new)
    }

    /// Gets the set of the stickers in a guild.
    ///
    /// This is an O(m) operation, where m is the amount of stickers in the
    /// guild. This requires the [`GUILDS`] and [`GUILD_EMOJIS_AND_STICKERS`]
    /// intents and the [`STICKER`] resource type.
    ///
    /// [`GUILDS`]: twilight_model::gateway::Intents::GUILDS
    /// [`GUILD_EMOJIS_AND_STICKERS`]: ::twilight_model::gateway::Intents::GUILD_EMOJIS_AND_STICKERS
    /// [`STICKER`]: crate::config::ResourceType::STICKER
    pub fn guild_stickers(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<StickerMarker>>>> {
        self.guild_stickers.get(&guild_id).map(Reference::new)
    }

    /// Gets the set of voice states in a guild.
    ///
    /// This requires both the [`GUILDS`] and [`GUILD_VOICE_STATES`] intents.
    ///
    /// [`GUILDS`]: ::twilight_model::gateway::Intents::GUILDS
    /// [`GUILD_VOICE_STATES`]: ::twilight_model::gateway::Intents::GUILD_VOICE_STATES
    pub fn guild_voice_states(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, Id<GuildMarker>, HashSet<Id<UserMarker>>>> {
        self.voice_state_guilds.get(&guild_id).map(Reference::new)
    }

    /// Gets an integration by guild ID and integration ID.
    ///
    /// This requires the [`GUILD_INTEGRATIONS`] intent. The
    /// [`ResourceType::INTEGRATION`] resource type must be enabled.
    ///
    /// [`GUILD_INTEGRATIONS`]: twilight_model::gateway::Intents::GUILD_INTEGRATIONS
    #[allow(clippy::type_complexity)]
    pub fn integration(
        &self,
        guild_id: Id<GuildMarker>,
        integration_id: Id<IntegrationMarker>,
    ) -> Option<
        Reference<'_, (Id<GuildMarker>, Id<IntegrationMarker>), GuildResource<GuildIntegration>>,
    > {
        self.integrations
            .get(&(guild_id, integration_id))
            .map(Reference::new)
    }

    /// Gets a member by guild ID and user ID.
    ///
    /// This requires the [`GUILD_MEMBERS`] intent.
    ///
    /// [`GUILD_MEMBERS`]: ::twilight_model::gateway::Intents::GUILD_MEMBERS
    #[allow(clippy::type_complexity)]
    pub fn member(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<Reference<'_, (Id<GuildMarker>, Id<UserMarker>), CachedMember>> {
        self.members.get(&(guild_id, user_id)).map(Reference::new)
    }

    /// Gets a message by ID.
    ///
    /// This requires one or both of the [`GUILD_MESSAGES`] or
    /// [`DIRECT_MESSAGES`] intents.
    ///
    /// [`GUILD_MESSAGES`]: ::twilight_model::gateway::Intents::GUILD_MESSAGES
    /// [`DIRECT_MESSAGES`]: ::twilight_model::gateway::Intents::DIRECT_MESSAGES
    pub fn message(
        &self,
        message_id: Id<MessageMarker>,
    ) -> Option<Reference<'_, Id<MessageMarker>, CachedMessage>> {
        self.messages.get(&message_id).map(Reference::new)
    }

    /// Gets a presence by, optionally, guild ID, and user ID.
    ///
    /// This requires the [`GUILD_PRESENCES`] intent.
    ///
    /// [`GUILD_PRESENCES`]: ::twilight_model::gateway::Intents::GUILD_PRESENCES
    #[allow(clippy::type_complexity)]
    pub fn presence(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<Reference<'_, (Id<GuildMarker>, Id<UserMarker>), CachedPresence>> {
        self.presences.get(&(guild_id, user_id)).map(Reference::new)
    }

    /// Gets a role by ID.
    ///
    /// This requires the [`GUILDS`] intent.
    ///
    /// [`GUILDS`]: ::twilight_model::gateway::Intents::GUILDS
    pub fn role(
        &self,
        role_id: Id<RoleMarker>,
    ) -> Option<Reference<'_, Id<RoleMarker>, GuildResource<Role>>> {
        self.roles.get(&role_id).map(Reference::new)
    }

    /// Gets a stage instance by ID.
    ///
    /// This requires the [`GUILDS`] intent.
    ///
    /// [`GUILDS`]: twilight_model::gateway::Intents::GUILDS
    pub fn stage_instance(
        &self,
        stage_id: Id<StageMarker>,
    ) -> Option<Reference<'_, Id<StageMarker>, GuildResource<StageInstance>>> {
        self.stage_instances.get(&stage_id).map(Reference::new)
    }

    /// Gets a sticker by ID.
    ///
    /// This is the O(1) operation. This requires the [`GUILDS`] and the
    /// [`GUILD_EMOJIS_AND_STICKERS`] intents and the [`STICKER`] resource type.
    ///
    /// [`GUILDS`]: twilight_model::gateway::Intents::GUILDS
    /// [`GUILD_EMOJIS_AND_STICKERS`]: ::twilight_model::gateway::Intents::GUILD_EMOJIS_AND_STICKERS
    /// [`STICKER`]: crate::config::ResourceType::STICKER
    pub fn sticker(
        &self,
        sticker_id: Id<StickerMarker>,
    ) -> Option<Reference<'_, Id<StickerMarker>, GuildResource<CachedSticker>>> {
        self.stickers.get(&sticker_id).map(Reference::new)
    }

    /// Gets a user by ID.
    ///
    /// This requires the [`GUILD_MEMBERS`] intent.
    ///
    /// [`GUILD_MEMBERS`]: ::twilight_model::gateway::Intents::GUILD_MEMBERS
    pub fn user(&self, user_id: Id<UserMarker>) -> Option<Reference<'_, Id<UserMarker>, User>> {
        self.users.get(&user_id).map(Reference::new)
    }

    /// Gets the voice states within a voice channel.
    ///
    /// This requires both the [`GUILDS`] and [`GUILD_VOICE_STATES`] intents.
    ///
    /// [`GUILDS`]: ::twilight_model::gateway::Intents::GUILDS
    /// [`GUILD_VOICE_STATES`]: ::twilight_model::gateway::Intents::GUILD_VOICE_STATES
    pub fn voice_channel_states(
        &self,
        channel_id: Id<ChannelMarker>,
    ) -> Option<VoiceChannelStates<'_>> {
        let user_ids = self.voice_state_channels.get(&channel_id)?;

        Some(VoiceChannelStates {
            index: 0,
            user_ids,
            voice_states: &self.voice_states,
        })
    }

    /// Gets a voice state by user ID and Guild ID.
    ///
    /// This requires both the [`GUILDS`] and [`GUILD_VOICE_STATES`] intents.
    ///
    /// [`GUILDS`]: ::twilight_model::gateway::Intents::GUILDS
    /// [`GUILD_VOICE_STATES`]: ::twilight_model::gateway::Intents::GUILD_VOICE_STATES
    #[allow(clippy::type_complexity)]
    pub fn voice_state(
        &self,
        user_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
    ) -> Option<Reference<'_, (Id<GuildMarker>, Id<UserMarker>), CachedVoiceState>> {
        self.voice_states
            .get(&(guild_id, user_id))
            .map(Reference::new)
    }

    /// Gets the highest role of a member.
    ///
    /// This requires both the [`GUILDS`] and [`GUILD_MEMBERS`] intents.
    ///
    /// [`GUILDS`]: twilight_model::gateway::Intents::GUILDS
    /// [`GUILD_MEMBERS`]: twilight_model::gateway::Intents::GUILD_MEMBERS
    pub fn member_highest_role(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<Id<RoleMarker>> {
        let member = self.members.get(&(guild_id, user_id))?;

        let mut highest_role: Option<(i64, Id<RoleMarker>)> = None;

        for role_id in &member.roles {
            if let Some(role) = self.role(*role_id) {
                if let Some((position, id)) = highest_role {
                    if role.position < position || (role.position == position && role.id > id) {
                        continue;
                    }
                }

                highest_role = Some((role.position, role.id));
            }
        }

        highest_role.map(|(_, id)| id)
    }

    fn new_with_config(config: Config) -> Self {
        Self {
            config,
            ..InMemoryCache::default()
        }
    }

    /// Determine whether the configured cache wants a specific resource to be
    /// processed.
    const fn wants(&self, resource_type: ResourceType) -> bool {
        self.config.resource_types().contains(resource_type)
    }
}

mod private {
    use twilight_model::gateway::{
        event::Event,
        payload::incoming::{
            ChannelCreate, ChannelDelete, ChannelPinsUpdate, ChannelUpdate, GuildCreate,
            GuildDelete, GuildEmojisUpdate, GuildStickersUpdate, GuildUpdate, IntegrationCreate,
            IntegrationDelete, IntegrationUpdate, InteractionCreate, MemberAdd, MemberChunk,
            MemberRemove, MemberUpdate, MessageCreate, MessageDelete, MessageDeleteBulk,
            MessageUpdate, PresenceUpdate, ReactionAdd, ReactionRemove, ReactionRemoveAll,
            ReactionRemoveEmoji, Ready, RoleCreate, RoleDelete, RoleUpdate, StageInstanceCreate,
            StageInstanceDelete, StageInstanceUpdate, ThreadCreate, ThreadDelete, ThreadListSync,
            ThreadUpdate, UnavailableGuild, UserUpdate, VoiceStateUpdate,
        },
    };

    pub trait Sealed {}

    impl Sealed for Event {}
    impl Sealed for ChannelCreate {}
    impl Sealed for ChannelDelete {}
    impl Sealed for ChannelPinsUpdate {}
    impl Sealed for ChannelUpdate {}
    impl Sealed for GuildCreate {}
    impl Sealed for GuildEmojisUpdate {}
    impl Sealed for GuildDelete {}
    impl Sealed for GuildStickersUpdate {}
    impl Sealed for GuildUpdate {}
    impl Sealed for IntegrationCreate {}
    impl Sealed for IntegrationDelete {}
    impl Sealed for IntegrationUpdate {}
    impl Sealed for InteractionCreate {}
    impl Sealed for MemberAdd {}
    impl Sealed for MemberChunk {}
    impl Sealed for MemberRemove {}
    impl Sealed for MemberUpdate {}
    impl Sealed for MessageCreate {}
    impl Sealed for MessageDelete {}
    impl Sealed for MessageDeleteBulk {}
    impl Sealed for MessageUpdate {}
    impl Sealed for PresenceUpdate {}
    impl Sealed for ReactionAdd {}
    impl Sealed for ReactionRemove {}
    impl Sealed for ReactionRemoveAll {}
    impl Sealed for ReactionRemoveEmoji {}
    impl Sealed for Ready {}
    impl Sealed for RoleCreate {}
    impl Sealed for RoleDelete {}
    impl Sealed for RoleUpdate {}
    impl Sealed for StageInstanceCreate {}
    impl Sealed for StageInstanceDelete {}
    impl Sealed for StageInstanceUpdate {}
    impl Sealed for ThreadCreate {}
    impl Sealed for ThreadDelete {}
    impl Sealed for ThreadListSync {}
    impl Sealed for ThreadUpdate {}
    impl Sealed for UnavailableGuild {}
    impl Sealed for UserUpdate {}
    impl Sealed for VoiceStateUpdate {}
}

/// Implemented for dispatch events.
///
/// This trait is sealed and cannot be implemented.
pub trait UpdateCache: private::Sealed {
    /// Updates the cache based on data contained within an event.
    // Allow this for presentation purposes in documentation.
    #[allow(unused_variables)]
    fn update(&self, cache: &InMemoryCache) {}
}

/// Iterator over a voice channel's list of voice states.
pub struct VoiceChannelStates<'a> {
    index: usize,
    #[allow(clippy::type_complexity)]
    user_ids: Ref<'a, Id<ChannelMarker>, HashSet<(Id<GuildMarker>, Id<UserMarker>)>>,
    voice_states: &'a DashMap<(Id<GuildMarker>, Id<UserMarker>), CachedVoiceState>,
}

impl<'a> Iterator for VoiceChannelStates<'a> {
    type Item = Reference<'a, (Id<GuildMarker>, Id<UserMarker>), CachedVoiceState>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((guild_id, user_id)) = self.user_ids.iter().nth(self.index) {
            if let Some(voice_state) = self.voice_states.get(&(*guild_id, *user_id)) {
                self.index += 1;

                return Some(Reference::new(voice_state));
            }
        }

        None
    }
}

impl UpdateCache for Event {
    // clippy: using `.deref()` is cleaner
    #[allow(clippy::cognitive_complexity, clippy::explicit_deref_methods)]
    fn update(&self, c: &InMemoryCache) {
        match self {
            Event::ChannelCreate(v) => c.update(v.deref()),
            Event::ChannelDelete(v) => c.update(v.deref()),
            Event::ChannelPinsUpdate(v) => c.update(v),
            Event::ChannelUpdate(v) => c.update(v.deref()),
            Event::GuildCreate(v) => c.update(v.deref()),
            Event::GuildDelete(v) => c.update(v),
            Event::GuildEmojisUpdate(v) => c.update(v),
            Event::GuildStickersUpdate(v) => c.update(v),
            Event::GuildUpdate(v) => c.update(v.deref()),
            Event::IntegrationCreate(v) => c.update(v.deref()),
            Event::IntegrationDelete(v) => c.update(v.deref()),
            Event::IntegrationUpdate(v) => c.update(v.deref()),
            Event::InteractionCreate(v) => c.update(v.deref()),
            Event::MemberAdd(v) => c.update(v.deref()),
            Event::MemberRemove(v) => c.update(v),
            Event::MemberUpdate(v) => c.update(v.deref()),
            Event::MemberChunk(v) => c.update(v),
            Event::MessageCreate(v) => c.update(v.deref()),
            Event::MessageDelete(v) => c.update(v),
            Event::MessageDeleteBulk(v) => c.update(v),
            Event::MessageUpdate(v) => c.update(v.deref()),
            Event::PresenceUpdate(v) => c.update(v.deref()),
            Event::ReactionAdd(v) => c.update(v.deref()),
            Event::ReactionRemove(v) => c.update(v.deref()),
            Event::ReactionRemoveAll(v) => c.update(v),
            Event::ReactionRemoveEmoji(v) => c.update(v),
            Event::Ready(v) => c.update(v.deref()),
            Event::RoleCreate(v) => c.update(v),
            Event::RoleDelete(v) => c.update(v),
            Event::RoleUpdate(v) => c.update(v),
            Event::StageInstanceCreate(v) => c.update(v),
            Event::StageInstanceDelete(v) => c.update(v),
            Event::StageInstanceUpdate(v) => c.update(v),
            Event::ThreadCreate(v) => c.update(v.deref()),
            Event::ThreadUpdate(v) => c.update(v.deref()),
            Event::ThreadDelete(v) => c.update(v),
            Event::ThreadListSync(v) => c.update(v),
            Event::UnavailableGuild(v) => c.update(v),
            Event::UserUpdate(v) => c.update(v),
            Event::VoiceStateUpdate(v) => c.update(v.deref()),
            // Ignored events.
            Event::AutoModerationActionExecution(_)
            | Event::AutoModerationRuleCreate(_)
            | Event::AutoModerationRuleDelete(_)
            | Event::AutoModerationRuleUpdate(_)
            | Event::BanAdd(_)
            | Event::BanRemove(_)
            | Event::CommandPermissionsUpdate(_)
            | Event::GatewayHeartbeat(_)
            | Event::GatewayHeartbeatAck
            | Event::GatewayHello(_)
            | Event::GatewayInvalidateSession(_)
            | Event::GatewayReconnect
            | Event::GiftCodeUpdate
            | Event::GuildIntegrationsUpdate(_)
            | Event::GuildScheduledEventCreate(_)
            | Event::GuildScheduledEventDelete(_)
            | Event::GuildScheduledEventUpdate(_)
            | Event::GuildScheduledEventUserAdd(_)
            | Event::GuildScheduledEventUserRemove(_)
            | Event::InviteCreate(_)
            | Event::InviteDelete(_)
            | Event::PresencesReplace
            | Event::Resumed
            | Event::ShardConnected(_)
            | Event::ShardConnecting(_)
            | Event::ShardDisconnected(_)
            | Event::ShardIdentifying(_)
            | Event::ShardPayload(_)
            | Event::ShardReconnecting(_)
            | Event::ShardResuming(_)
            | Event::ThreadMembersUpdate(_)
            | Event::ThreadMemberUpdate(_)
            | Event::TypingStart(_)
            | Event::VoiceServerUpdate(_)
            | Event::WebhooksUpdate(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{test, InMemoryCache};
    use twilight_model::{
        gateway::payload::incoming::RoleDelete,
        guild::{Member, Permissions, Role},
        id::Id,
        util::Timestamp,
    };

    #[test]
    fn syntax_update() {
        let cache = InMemoryCache::new();
        cache.update(&RoleDelete {
            guild_id: Id::new(1),
            role_id: Id::new(1),
        });
    }

    #[test]
    fn clear() {
        let cache = InMemoryCache::new();
        cache.cache_emoji(Id::new(1), test::emoji(Id::new(3), None));
        cache.cache_member(Id::new(2), test::member(Id::new(4), Id::new(2)));
        cache.clear();
        assert!(cache.emojis.is_empty());
        assert!(cache.members.is_empty());
    }

    #[test]
    fn highest_role() {
        let joined_at = Timestamp::from_secs(1_632_072_645).expect("non zero");
        let cache = InMemoryCache::new();
        let guild_id = Id::new(1);
        let user = test::user(Id::new(1));
        cache.cache_member(
            guild_id,
            Member {
                avatar: None,
                communication_disabled_until: None,
                deaf: false,
                guild_id,
                joined_at,
                mute: false,
                nick: None,
                pending: false,
                premium_since: None,
                roles: vec![Id::new(1), Id::new(2)],
                user,
            },
        );

        cache.cache_roles(
            guild_id,
            vec![
                Role {
                    color: 0,
                    hoist: false,
                    icon: None,
                    id: Id::new(1),
                    managed: false,
                    mentionable: false,
                    name: "test".to_owned(),
                    permissions: Permissions::empty(),
                    position: 0,
                    tags: None,
                    unicode_emoji: None,
                },
                Role {
                    color: 0,
                    hoist: false,
                    icon: None,
                    id: Id::new(2),
                    managed: false,
                    mentionable: false,
                    name: "test".to_owned(),
                    permissions: Permissions::empty(),
                    position: 1,
                    tags: None,
                    unicode_emoji: None,
                },
            ],
        );

        assert_eq!(
            cache.member_highest_role(guild_id, Id::new(1)),
            Some(Id::new(2))
        );
    }
}
