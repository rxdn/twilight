use bitflags::bitflags;
use twilight_model::gateway::event::EventType;

bitflags! {
    /// Bitflags representing all of the possible types of events.
    pub struct EventTypeFlags: u128 {
        /// Message has been blocked by AutoMod according to a rule.
        const AUTO_MODERATION_ACTION_EXECUTION = 1 << 71;
        /// [`AutoModerationRule`] has been created.
        ///
        /// [`AutoModerationRule`]: crate::guild::auto_moderation::AutoModerationRule
        const AUTO_MODERATION_RULE_CREATE = 1 << 72;
        /// [`AutoModerationRule`] has been deleted.
        ///
        /// [`AutoModerationRule`]: crate::guild::auto_moderation::AutoModerationRule
        const AUTO_MODERATION_RULE_DELETE = 1 << 73;
        /// [`AutoModerationRule`] has been updated.
        ///
        /// [`AutoModerationRule`]: crate::guild::auto_moderation::AutoModerationRule
        const AUTO_MODERATION_RULE_UPDATE = 1 << 74;
        /// User has been banned from a guild.
        const BAN_ADD = 1;
        /// User has been unbanned from a guild.
        const BAN_REMOVE = 1 << 1;
        /// Channel has been created.
        const CHANNEL_CREATE = 1 << 2;
        /// Channel has been deleted.
        const CHANNEL_DELETE = 1 << 3;
        /// Channel's pins have been updated.
        const CHANNEL_PINS_UPDATE = 1 << 4;
        /// Channel has been updated.
        const CHANNEL_UPDATE = 1 << 5;
        /// A command's permissions has been updated.
        const COMMAND_PERMISSIONS_UPDATE = 1 << 70;
        /// Heartbeat has been created.
        const GATEWAY_HEARTBEAT = 1 << 6;
        /// Heartbeat has been acknowledged.
        const GATEWAY_HEARTBEAT_ACK = 1 << 7;
        /// A "hello" packet has been received from the gateway.
        const GATEWAY_HELLO = 1 << 8;
        /// Shard's session has been invalidated.
        ///
        /// A payload containing a boolean is included. If `true` the session is
        /// resumable. If not, then the shard must initialize a new session.
        const GATEWAY_INVALIDATE_SESSION = 1 << 69;
        /// Gateway is indicating that a shard should perform a reconnect.
        const GATEWAY_RECONNECT = 1 << 9;
        /// Gift code sent in a channel has been updated.
        const GIFT_CODE_UPDATE = 1 << 49;
        /// A guild has been created.
        const GUILD_CREATE = 1 << 10;
        /// A guild has been deleted or the current user has been removed from a guild.
        const GUILD_DELETE = 1 << 11;
        /// A guild's emojis have been updated.
        const GUILD_EMOJIS_UPDATE = 1 << 12;
        /// A guild's integrations have been updated.
        const GUILD_INTEGRATIONS_UPDATE = 1 << 13;
        /// A guild's integrations have been updated.
        const GUILD_SCHEDULED_EVENT_CREATE = 1 << 64;
        /// A guild's integrations have been updated.
        const GUILD_SCHEDULED_EVENT_DELETE = 1 << 65;
        /// A guild's integrations have been updated.
        const GUILD_SCHEDULED_EVENT_UPDATE = 1 << 66;
        /// A guild's integrations have been updated.
        const GUILD_SCHEDULED_EVENT_USER_ADD = 1 << 67;
        /// A guild's integrations have been updated.
        const GUILD_SCHEDULED_EVENT_USER_REMOVE = 1 << 68;
        /// A guild's stickers have been updated.
        const GUILD_STICKERS_UPDATE = 1 << 63;
        /// A guild has been updated.
        const GUILD_UPDATE = 1 << 14;
        /// A guild integration was created.
        const INTEGRATION_CREATE = 1 << 60;
        /// A guild integration was deleted.
        const INTEGRATION_DELETE = 1 << 61;
        /// A guild integration was updated.
        const INTEGRATION_UPDATE = 1 << 62;
        /// An interaction was invoked by a user.
        const INTERACTION_CREATE = 1 << 56;
        /// Invite for a channel has been created.
        const INVITE_CREATE = 1 << 46;
        /// Invite for a channel has been deleted.
        const INVITE_DELETE = 1 << 47;
        /// Member has been added to a guild.
        const MEMBER_ADD = 1 << 15;
        /// Member has been removed from a guild.
        const MEMBER_REMOVE = 1 << 16;
        /// Member in a guild has been updated.
        const MEMBER_UPDATE = 1 << 17;
        /// Group of members from a guild.
        ///
        /// This may be all of the remaining members or not; the chunk index in
        /// the event payload needs to be confirmed.
        const MEMBER_CHUNK = 1 << 18;
        /// Message created in a channel.
        const MESSAGE_CREATE = 1 << 19;
        /// Message deleted in a channel.
        const MESSAGE_DELETE = 1 << 20;
        /// Multiple messages have been deleted in a channel.
        const MESSAGE_DELETE_BULK = 1 << 21;
        /// Message in a channel has been updated.
        const MESSAGE_UPDATE = 1 << 22;
        /// User's presence details are updated.
        const PRESENCE_UPDATE = 1 << 23;
        /// Group of presences are replaced.
        ///
        /// This is a placeholder as it *can* happen for bots but has no real
        /// meaning.
        const PRESENCES_REPLACE = 1 << 24;
        /// Reaction has been added to a message.
        const REACTION_ADD = 1 << 25;
        /// Reaction has been removed from a message.
        const REACTION_REMOVE = 1 << 26;
        /// All of the reactions for a message have been removed.
        const REACTION_REMOVE_ALL = 1 << 27;
        /// All of a given emoji's reactions for a message have been removed.
        const REACTION_REMOVE_EMOJI = 1 << 48;
        /// Session is initialized.
        const READY = 1 << 28;
        /// Session is resumed.
        const RESUMED = 1 << 29;
        /// Role has been created in a guild.
        const ROLE_CREATE = 1 << 30;
        /// Role has been deleted in a guild.
        const ROLE_DELETE = 1 << 31;
        /// Role has been updated in a guild.
        const ROLE_UPDATE = 1 << 32;
        /// Shard has finalized a session with the gateway.
        const SHARD_CONNECTED = 1 << 33;
        /// Shard has begun connecting to the gateway.
        const SHARD_CONNECTING = 1 << 34;
        /// Shard has disconnected from the gateway.
        const SHARD_DISCONNECTED = 1 << 35;
        /// Shard is identifying to create a session with the gateway.
        const SHARD_IDENTIFYING = 1 << 36;
        /// Incoming message has been received from the gateway.
        const SHARD_PAYLOAD = 1 << 45;
        /// Shard is reconnecting to the gateway.
        const SHARD_RECONNECTING = 1 << 37;
        /// Shard is resuming a session with the gateway.
        const SHARD_RESUMING = 1 << 38;
        /// Stage instance was created in a stage channel.
        const STAGE_INSTANCE_CREATE = 1 << 57;
        /// Stage instance was deleted in a stage channel.
        const STAGE_INSTANCE_DELETE = 1 << 58;
        /// Stage instance was updated in a stage channel.
        const STAGE_INSTANCE_UPDATE = 1 << 59;
        /// A thread has been created, relevant to the current user,
        /// or the current user has been added to a thread.
        const THREAD_CREATE = 1 << 50;
        /// A thread, relevant to the current user, has been deleted.
        const THREAD_DELETE = 1 << 52;
        /// The current user has gained access to a thread.
        const THREAD_LIST_SYNC = 1 << 53;
        /// A user has been added to or removed from a thread.
        const THREAD_MEMBERS_UPDATE = 1 << 55;
        /// The thread member object for the current user has been updated.
        const THREAD_MEMBER_UPDATE = 1 << 54;
        /// A thread has been updated.
        const THREAD_UPDATE = 1 << 51;
        /// User has begun typing in a channel.
        const TYPING_START = 1 << 39;
        /// Guild is unavailable, potentially due to an outage.
        const UNAVAILABLE_GUILD = 1 << 40;
        /// Current user's profile has been updated.
        const USER_UPDATE = 1 << 41;
        /// Voice server has provided an update with voice session details.
        const VOICE_SERVER_UPDATE = 1 << 42;
        /// User's state in a voice channel has been updated.
        const VOICE_STATE_UPDATE = 1 << 43;
        /// Webhook in a guild has been updated.
        const WEBHOOKS_UPDATE = 1 << 44;

        /// All [`EventTypeFlags`] in [`Intents::AUTO_MODERATION_CONFIGURATION`].
        ///
        /// [`Intents::AUTO_MODERATION_CONFIGURATION`]: crate::Intents::AUTO_MODERATION_CONFIGURATION
        const AUTO_MODERATION_CONFIGURATION = Self::AUTO_MODERATION_RULE_CREATE.bits()
                | Self::AUTO_MODERATION_RULE_DELETE.bits()
                | Self::AUTO_MODERATION_RULE_UPDATE.bits();
        /// All [`EventTypeFlags`] in [`Intents::AUTO_MODERATION_EXECUTION`].
        ///
        /// [`Intents::AUTO_MODERATION_EXECUTION`]: crate::Intents::AUTO_MODERATION_EXECUTION
        const AUTO_MODERATION_EXECUTION = Self::AUTO_MODERATION_ACTION_EXECUTION.bits();
        /// All [`EventTypeFlags`] in [`Intents::DIRECT_MESSAGES`].
        ///
        /// [`Intents::DIRECT_MESSAGES`]: crate::Intents::DIRECT_MESSAGES
        const DIRECT_MESSAGES = Self::MESSAGE_CREATE.bits()
            | Self::MESSAGE_DELETE.bits()
            | Self::MESSAGE_DELETE_BULK.bits()
            | Self::MESSAGE_UPDATE.bits();
        /// All [`EventTypeFlags`] in [`Intents::DIRECT_MESSAGE_REACTIONS`].
        ///
        /// [`Intents::DIRECT_MESSAGE_REACTIONS`]: crate::Intents::DIRECT_MESSAGE_REACTIONS
        const DIRECT_MESSAGE_REACTIONS = Self::REACTION_ADD.bits()
            | Self::REACTION_REMOVE.bits()
            | Self::REACTION_REMOVE_ALL.bits()
            | Self::REACTION_REMOVE_EMOJI.bits();
        /// All [`EventTypeFlags`] in [`Intents::DIRECT_MESSAGE_TYPING`].
        ///
        /// [`Intents::DIRECT_MESSAGE_TYPING`]: crate::Intents::DIRECT_MESSAGE_TYPING
        const DIRECT_MESSAGE_TYPING = Self::TYPING_START.bits();
        /// All [`EventTypeFlags`] in [`Intents::GUILDS`].
        ///
        /// [`Intents::GUILDS`]: crate::Intents::GUILDS
        const GUILDS = Self::CHANNEL_CREATE.bits()
            | Self::CHANNEL_DELETE.bits()
            | Self::CHANNEL_PINS_UPDATE.bits()
            | Self::CHANNEL_UPDATE.bits()
            | Self::GUILD_CREATE.bits()
            | Self::GUILD_DELETE.bits()
            | Self::GUILD_UPDATE.bits()
            | Self::ROLE_CREATE.bits()
            | Self::ROLE_DELETE.bits()
            | Self::ROLE_UPDATE.bits()
            | Self::STAGE_INSTANCE_CREATE.bits()
            | Self::STAGE_INSTANCE_UPDATE.bits()
            | Self::STAGE_INSTANCE_DELETE.bits()
            | Self::THREAD_CREATE.bits()
            | Self::THREAD_UPDATE.bits()
            | Self::THREAD_DELETE.bits()
            | Self::THREAD_LIST_SYNC.bits()
            | Self::THREAD_MEMBER_UPDATE.bits()
            | Self::THREAD_MEMBERS_UPDATE.bits();
        /// All [`EventTypeFlags`] in [`Intents::GUILD_BANS`].
        ///
        /// [`Intents::GUILD_BANS`]: crate::Intents::GUILD_BANS
        const GUILD_BANS = Self::BAN_ADD.bits() | Self::BAN_REMOVE.bits();
        /// All [`EventTypeFlags`] in [`Intents::GUILD_EMOJIS_AND_STICKERS`].
        ///
        /// [`Intents::GUILD_EMOJIS_AND_STICKERS`]: crate::Intents::GUILD_EMOJIS_AND_STICKERS
        const GUILD_EMOJIS_AND_STICKERS = Self::GUILD_EMOJIS_UPDATE.bits()
            | Self::GUILD_STICKERS_UPDATE.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_INTEGRATIONS`].
        ///
        /// [`Intents::GUILD_INTEGRATIONS`]: crate::Intents::GUILD_INTEGRATIONS
        const GUILD_INTEGRATIONS = Self::GUILD_INTEGRATIONS_UPDATE.bits()
            | Self::INTEGRATION_CREATE.bits()
            | Self::INTEGRATION_UPDATE.bits()
            | Self::INTEGRATION_DELETE.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_INVITES`].
        ///
        /// [`Intents::GUILD_INVITES`]: crate::Intents::GUILD_INVITES
        const GUILD_INVITES = Self::INVITE_CREATE.bits() | Self::INVITE_DELETE.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_MEMBERS`].
        ///
        /// [`Intents::GUILD_MEMBERS`]: crate::Intents::GUILD_MEMBERS
        const GUILD_MEMBERS = Self::MEMBER_ADD.bits()
            | Self::MEMBER_REMOVE.bits()
            | Self::MEMBER_UPDATE.bits()
            | Self::THREAD_MEMBERS_UPDATE.bits();


        /// All [`EventTypeFlags`] in [`Intents::GUILD_MESSAGES`].
        ///
        /// [`Intents::GUILD_MESSAGES`]: crate::Intents::GUILD_MESSAGES
        const GUILD_MESSAGES = Self::MESSAGE_CREATE.bits()
            | Self::MESSAGE_DELETE.bits()
            | Self::MESSAGE_DELETE.bits()
            | Self::MESSAGE_DELETE_BULK.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_MESSAGE_REACTIONS`].
        ///
        /// [`Intents::GUILD_MESSAGE_REACTIONS`]: crate::Intents::GUILD_MESSAGE_REACTIONS
        const GUILD_MESSAGE_REACTIONS = Self::REACTION_ADD.bits()
            | Self::REACTION_REMOVE.bits()
            | Self::REACTION_REMOVE_ALL.bits()
            | Self::REACTION_REMOVE_EMOJI.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_MESSAGE_TYPING`].
        ///
        /// [`Intents::GUILD_MESSAGE_TYPING`]: crate::Intents::GUILD_MESSAGE_TYPING
        const GUILD_MESSAGE_TYPING = Self::TYPING_START.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_PRESENCES`].
        ///
        /// [`Intents::GUILD_PRESENCES`]: crate::Intents::GUILD_PRESENCES
        const GUILD_PRESENCES = Self::PRESENCE_UPDATE.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_SCHEDULED_EVENTS`].
        ///
        /// [`Intents::GUILD_SCHEDULED_EVENTS`]: crate::Intents::GUILD_SCHEDULED_EVENTS
        const GUILD_SCHEDULED_EVENTS = Self::GUILD_SCHEDULED_EVENT_CREATE.bits()
            | Self::GUILD_SCHEDULED_EVENT_DELETE.bits()
            | Self::GUILD_SCHEDULED_EVENT_UPDATE.bits()
            | Self::GUILD_SCHEDULED_EVENT_USER_ADD.bits()
            | Self::GUILD_SCHEDULED_EVENT_USER_REMOVE.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_VOICE_STATES`].
        ///
        /// [`Intents::GUILD_VOICE_STATES`]: crate::Intents::GUILD_VOICE_STATES
        const GUILD_VOICE_STATES = Self::VOICE_STATE_UPDATE.bits();

        /// All [`EventTypeFlags`] in [`Intents::GUILD_WEBHOOKS`].
        ///
        /// [`Intents::GUILD_WEBHOOKS`]: crate::Intents::GUILD_WEBHOOKS
        const GUILD_WEBHOOKS = Self::WEBHOOKS_UPDATE.bits();

    }
}

impl From<EventType> for EventTypeFlags {
    fn from(event_type: EventType) -> Self {
        match event_type {
            EventType::AutoModerationActionExecution => Self::AUTO_MODERATION_ACTION_EXECUTION,
            EventType::AutoModerationRuleCreate => Self::AUTO_MODERATION_RULE_CREATE,
            EventType::AutoModerationRuleDelete => Self::AUTO_MODERATION_RULE_DELETE,
            EventType::AutoModerationRuleUpdate => Self::AUTO_MODERATION_RULE_UPDATE,
            EventType::BanAdd => Self::BAN_ADD,
            EventType::BanRemove => Self::BAN_REMOVE,
            EventType::ChannelCreate => Self::CHANNEL_CREATE,
            EventType::ChannelDelete => Self::CHANNEL_DELETE,
            EventType::ChannelPinsUpdate => Self::CHANNEL_PINS_UPDATE,
            EventType::ChannelUpdate => Self::CHANNEL_UPDATE,
            EventType::CommandPermissionsUpdate => Self::COMMAND_PERMISSIONS_UPDATE,
            EventType::GatewayHeartbeat => Self::GATEWAY_HEARTBEAT,
            EventType::GatewayHeartbeatAck => Self::GATEWAY_HEARTBEAT_ACK,
            EventType::GatewayHello => Self::GATEWAY_HELLO,
            EventType::GatewayInvalidateSession => Self::GATEWAY_INVALIDATE_SESSION,
            EventType::GatewayReconnect => Self::GATEWAY_RECONNECT,
            EventType::GiftCodeUpdate => Self::GIFT_CODE_UPDATE,
            EventType::GuildCreate => Self::GUILD_CREATE,
            EventType::GuildDelete => Self::GUILD_DELETE,
            EventType::GuildEmojisUpdate => Self::GUILD_EMOJIS_UPDATE,
            EventType::GuildIntegrationsUpdate => Self::GUILD_INTEGRATIONS_UPDATE,
            EventType::GuildScheduledEventCreate => Self::GUILD_SCHEDULED_EVENT_CREATE,
            EventType::GuildScheduledEventDelete => Self::GUILD_SCHEDULED_EVENT_DELETE,
            EventType::GuildScheduledEventUpdate => Self::GUILD_SCHEDULED_EVENT_UPDATE,
            EventType::GuildScheduledEventUserAdd => Self::GUILD_SCHEDULED_EVENT_USER_ADD,
            EventType::GuildScheduledEventUserRemove => Self::GUILD_SCHEDULED_EVENT_USER_REMOVE,
            EventType::GuildStickersUpdate => Self::GUILD_STICKERS_UPDATE,
            EventType::GuildUpdate => Self::GUILD_UPDATE,
            EventType::IntegrationCreate => Self::INTEGRATION_CREATE,
            EventType::IntegrationDelete => Self::INTEGRATION_DELETE,
            EventType::IntegrationUpdate => Self::INTEGRATION_UPDATE,
            EventType::InteractionCreate => Self::INTERACTION_CREATE,
            EventType::InviteCreate => Self::INVITE_CREATE,
            EventType::InviteDelete => Self::INVITE_DELETE,
            EventType::MemberAdd => Self::MEMBER_ADD,
            EventType::MemberRemove => Self::MEMBER_REMOVE,
            EventType::MemberUpdate => Self::MEMBER_UPDATE,
            EventType::MemberChunk => Self::MEMBER_CHUNK,
            EventType::MessageCreate => Self::MESSAGE_CREATE,
            EventType::MessageDelete => Self::MESSAGE_DELETE,
            EventType::MessageDeleteBulk => Self::MESSAGE_DELETE_BULK,
            EventType::MessageUpdate => Self::MESSAGE_UPDATE,
            EventType::PresenceUpdate => Self::PRESENCE_UPDATE,
            EventType::PresencesReplace => Self::PRESENCES_REPLACE,
            EventType::ReactionAdd => Self::REACTION_ADD,
            EventType::ReactionRemove => Self::REACTION_REMOVE,
            EventType::ReactionRemoveAll => Self::REACTION_REMOVE_ALL,
            EventType::ReactionRemoveEmoji => Self::REACTION_REMOVE_EMOJI,
            EventType::Ready => Self::READY,
            EventType::Resumed => Self::RESUMED,
            EventType::RoleCreate => Self::ROLE_CREATE,
            EventType::RoleDelete => Self::ROLE_DELETE,
            EventType::RoleUpdate => Self::ROLE_UPDATE,
            EventType::ShardConnected => Self::SHARD_CONNECTED,
            EventType::ShardConnecting => Self::SHARD_CONNECTING,
            EventType::ShardDisconnected => Self::SHARD_DISCONNECTED,
            EventType::ShardIdentifying => Self::SHARD_IDENTIFYING,
            EventType::ShardReconnecting => Self::SHARD_RECONNECTING,
            EventType::ShardPayload => Self::SHARD_PAYLOAD,
            EventType::ShardResuming => Self::SHARD_RESUMING,
            EventType::StageInstanceCreate => Self::STAGE_INSTANCE_CREATE,
            EventType::StageInstanceDelete => Self::STAGE_INSTANCE_DELETE,
            EventType::StageInstanceUpdate => Self::STAGE_INSTANCE_UPDATE,
            EventType::ThreadCreate => Self::THREAD_CREATE,
            EventType::ThreadDelete => Self::THREAD_DELETE,
            EventType::ThreadListSync => Self::THREAD_LIST_SYNC,
            EventType::ThreadMembersUpdate => Self::THREAD_MEMBERS_UPDATE,
            EventType::ThreadMemberUpdate => Self::THREAD_MEMBER_UPDATE,
            EventType::ThreadUpdate => Self::THREAD_UPDATE,
            EventType::TypingStart => Self::TYPING_START,
            EventType::UnavailableGuild => Self::UNAVAILABLE_GUILD,
            EventType::UserUpdate => Self::USER_UPDATE,
            EventType::VoiceServerUpdate => Self::VOICE_SERVER_UPDATE,
            EventType::VoiceStateUpdate => Self::VOICE_STATE_UPDATE,
            EventType::WebhooksUpdate => Self::WEBHOOKS_UPDATE,
        }
    }
}

impl<'a> TryFrom<(u8, Option<&'a str>)> for EventTypeFlags {
    type Error = (u8, Option<&'a str>);

    fn try_from((op, event_type): (u8, Option<&'a str>)) -> Result<Self, Self::Error> {
        match (op, event_type) {
            (1, _) => Ok(Self::GATEWAY_HEARTBEAT),
            (7, _) => Ok(Self::GATEWAY_RECONNECT),
            (9, _) => Ok(Self::GATEWAY_INVALIDATE_SESSION),
            (10, _) => Ok(Self::GATEWAY_HELLO),
            (11, _) => Ok(Self::GATEWAY_HEARTBEAT_ACK),
            (_, Some(event_type)) => {
                let flag = EventType::try_from(event_type).map_err(|kind| (op, Some(kind)))?;

                Ok(Self::from(flag))
            }
            (_, None) => Err((op, event_type)),
        }
    }
}

impl Default for EventTypeFlags {
    fn default() -> Self {
        let mut flags = Self::all();
        flags.remove(Self::SHARD_PAYLOAD);

        flags
    }
}

#[cfg(test)]
mod tests {
    use super::{EventType, EventTypeFlags};
    use static_assertions::assert_impl_all;
    use std::{fmt::Debug, hash::Hash};

    assert_impl_all!(
        EventTypeFlags: Copy,
        Clone,
        Debug,
        Eq,
        From<EventType>,
        Hash,
        PartialEq,
        Send,
        Sync,
        TryFrom<(u8, Option<&'static str>)>
    );
}
