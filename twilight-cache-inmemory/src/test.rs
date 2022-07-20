use crate::InMemoryCache;
use twilight_model::{
    channel::message::sticker::{Sticker, StickerFormatType, StickerType},
    id::marker::StickerMarker,
};
use twilight_model::{
    channel::{
        message::{Message, MessageFlags, MessageType},
        Channel, ChannelType, Reaction, ReactionType,
    },
    gateway::payload::incoming::{MessageCreate, ReactionAdd},
    guild::{
        DefaultMessageNotificationLevel, Emoji, ExplicitContentFilter, Guild, Member, MfaLevel,
        NSFWLevel, PartialMember, Permissions, PremiumTier, Role, SystemChannelFlags,
        VerificationLevel,
    },
    id::{
        marker::{ChannelMarker, EmojiMarker, GuildMarker, RoleMarker, UserMarker},
        Id,
    },
    user::{CurrentUser, User},
    util::{ImageHash, Timestamp},
    voice::VoiceState,
};

pub fn cache() -> InMemoryCache {
    InMemoryCache::new()
}

#[allow(clippy::too_many_lines)]
pub fn cache_with_message_and_reactions() -> InMemoryCache {
    let joined_at = Timestamp::from_secs(1_632_072_645).expect("non zero");
    let cache = InMemoryCache::new();
    let avatar = ImageHash::parse(b"6961d9f1fdb5880bf4a3ec6348d3bbcf").unwrap();

    let msg = Message {
        activity: None,
        application: None,
        application_id: None,
        attachments: Vec::new(),
        author: User {
            accent_color: None,
            avatar: Some(avatar),
            banner: None,
            bot: false,
            discriminator: 1,
            email: None,
            flags: None,
            id: Id::new(3),
            locale: None,
            mfa_enabled: None,
            name: "test".to_owned(),
            premium_type: None,
            public_flags: None,
            system: None,
            verified: None,
        },
        channel_id: Id::new(2),
        components: Vec::new(),
        content: "ping".to_owned(),
        edited_timestamp: None,
        embeds: Vec::new(),
        flags: Some(MessageFlags::empty()),
        guild_id: Some(Id::new(1)),
        id: Id::new(4),
        interaction: None,
        kind: MessageType::Regular,
        member: Some(PartialMember {
            avatar: None,
            communication_disabled_until: None,
            deaf: false,
            joined_at,
            mute: false,
            nick: Some("member nick".to_owned()),
            permissions: None,
            premium_since: None,
            roles: Vec::new(),
            user: None,
        }),
        mention_channels: Vec::new(),
        mention_everyone: false,
        mention_roles: Vec::new(),
        mentions: Vec::new(),
        pinned: false,
        reactions: Vec::new(),
        reference: None,
        sticker_items: Vec::new(),
        thread: None,
        referenced_message: None,
        timestamp: Timestamp::from_secs(1_632_072_645).expect("non zero"),
        tts: false,
        webhook_id: None,
    };

    cache.update(&MessageCreate(msg));

    let mut reaction = ReactionAdd(Reaction {
        channel_id: Id::new(2),
        emoji: ReactionType::Unicode {
            name: "😀".to_owned(),
        },
        guild_id: Some(Id::new(1)),
        member: Some(Member {
            avatar: None,
            communication_disabled_until: None,
            deaf: false,
            guild_id: Id::new(1),
            joined_at,
            mute: false,
            nick: Some("member nick".to_owned()),
            pending: false,
            premium_since: None,
            roles: Vec::new(),
            user: User {
                accent_color: None,
                avatar: Some(avatar),
                banner: None,
                bot: false,
                discriminator: 1,
                email: None,
                flags: None,
                id: Id::new(3),
                locale: None,
                mfa_enabled: None,
                name: "test".to_owned(),
                premium_type: None,
                public_flags: None,
                system: None,
                verified: None,
            },
        }),
        message_id: Id::new(4),
        user_id: Id::new(3),
    });

    cache.update(&reaction);

    let user_5_input = b"ef678abdee09d8dfb14e83381983d5e4";
    let user_5_avatar = ImageHash::parse(user_5_input).unwrap();

    reaction.member.replace(Member {
        avatar: None,
        communication_disabled_until: None,
        deaf: false,
        guild_id: Id::new(1),
        joined_at,
        mute: false,
        nick: None,
        pending: false,
        premium_since: None,
        roles: Vec::new(),
        user: User {
            accent_color: None,
            avatar: Some(user_5_avatar),
            banner: None,
            bot: false,
            discriminator: 2,
            email: None,
            flags: None,
            id: Id::new(5),
            locale: None,
            mfa_enabled: None,
            name: "test".to_owned(),
            premium_type: None,
            public_flags: None,
            system: None,
            verified: None,
        },
    });
    reaction.user_id = Id::new(5);

    cache.update(&reaction);

    reaction.emoji = ReactionType::Unicode {
        name: "🗺️".to_owned(),
    };

    cache.update(&reaction);

    reaction.emoji = ReactionType::Custom {
        animated: true,
        id: Id::new(6),
        name: Some("custom".to_owned()),
    };

    cache.update(&reaction);

    cache
}

pub fn current_user(id: u64) -> CurrentUser {
    CurrentUser {
        accent_color: Some(0xFF_00_00),
        avatar: None,
        banner: None,
        bot: true,
        discriminator: 9876,
        email: None,
        id: Id::new(id),
        mfa_enabled: true,
        name: "test".to_owned(),
        verified: Some(true),
        premium_type: None,
        public_flags: None,
        flags: None,
        locale: None,
    }
}

pub fn emoji(id: Id<EmojiMarker>, user: Option<User>) -> Emoji {
    Emoji {
        animated: false,
        available: true,
        id,
        managed: false,
        name: "test".to_owned(),
        require_colons: true,
        roles: Vec::new(),
        user,
    }
}

pub fn guild_channel_text() -> (Id<GuildMarker>, Id<ChannelMarker>, Channel) {
    let guild_id = Id::new(1);
    let channel_id = Id::new(2);
    let channel = Channel {
        application_id: None,
        bitrate: None,
        default_auto_archive_duration: None,
        guild_id: Some(guild_id),
        icon: None,
        id: channel_id,
        invitable: None,
        kind: ChannelType::GuildText,
        last_message_id: None,
        last_pin_timestamp: None,
        member: None,
        member_count: None,
        message_count: None,
        name: Some("test".to_owned()),
        newly_created: None,
        nsfw: Some(false),
        owner_id: None,
        parent_id: None,
        permission_overwrites: Some(Vec::new()),
        position: Some(3),
        rate_limit_per_user: None,
        recipients: None,
        rtc_region: None,
        thread_metadata: None,
        topic: None,
        user_limit: None,
        video_quality_mode: None,
    };

    (guild_id, channel_id, channel)
}

pub fn member(id: Id<UserMarker>, guild_id: Id<GuildMarker>) -> Member {
    let joined_at = Timestamp::from_secs(1_632_072_645).expect("non zero");

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
        roles: Vec::new(),
        user: user(id),
    }
}

pub fn role(id: Id<RoleMarker>) -> Role {
    Role {
        color: 0,
        hoist: false,
        icon: None,
        id,
        managed: false,
        mentionable: false,
        name: "test".to_owned(),
        permissions: Permissions::empty(),
        position: 0,
        tags: None,
        unicode_emoji: None,
    }
}

pub const fn sticker(id: Id<StickerMarker>, guild_id: Id<GuildMarker>) -> Sticker {
    Sticker {
        available: false,
        description: None,
        format_type: StickerFormatType::Png,
        guild_id: Some(guild_id),
        id,
        kind: StickerType::Standard,
        name: String::new(),
        pack_id: None,
        sort_value: None,
        tags: String::new(),
        user: None,
    }
}

pub fn voice_state(
    guild_id: Id<GuildMarker>,
    channel_id: Option<Id<ChannelMarker>>,
    user_id: Id<UserMarker>,
) -> VoiceState {
    VoiceState {
        channel_id,
        deaf: false,
        guild_id: Some(guild_id),
        member: None,
        mute: true,
        self_deaf: false,
        self_mute: true,
        self_stream: false,
        self_video: false,
        session_id: "a".to_owned(),
        suppress: false,
        token: None,
        user_id,
        request_to_speak_timestamp: Some(Timestamp::from_secs(1_632_072_645).expect("non zero")),
    }
}

pub fn user(id: Id<UserMarker>) -> User {
    let banner_hash = b"16ed037ab6dae5e1739f15c745d12454";
    let banner = ImageHash::parse(banner_hash).expect("valid hash");

    User {
        accent_color: None,
        avatar: None,
        banner: Some(banner),
        bot: false,
        discriminator: 1,
        email: None,
        flags: None,
        id,
        locale: None,
        mfa_enabled: None,
        name: "user".to_owned(),
        premium_type: None,
        public_flags: None,
        system: None,
        verified: None,
    }
}

pub fn guild(id: Id<GuildMarker>, member_count: Option<u64>) -> Guild {
    Guild {
        afk_channel_id: None,
        afk_timeout: 0,
        application_id: None,
        approximate_member_count: None,
        approximate_presence_count: None,
        banner: None,
        channels: Vec::new(),
        default_message_notifications: DefaultMessageNotificationLevel::Mentions,
        description: None,
        discovery_splash: None,
        emojis: Vec::new(),
        explicit_content_filter: ExplicitContentFilter::None,
        features: Vec::new(),
        icon: None,
        id,
        joined_at: None,
        large: false,
        max_members: None,
        max_presences: None,
        max_video_channel_users: None,
        member_count,
        members: Vec::new(),
        mfa_level: MfaLevel::None,
        name: "test".to_owned(),
        nsfw_level: NSFWLevel::Default,
        owner_id: Id::new(1),
        owner: None,
        permissions: None,
        preferred_locale: "en_us".to_owned(),
        premium_progress_bar_enabled: false,
        premium_subscription_count: None,
        premium_tier: PremiumTier::None,
        presences: Vec::new(),
        roles: Vec::new(),
        rules_channel_id: None,
        splash: None,
        stage_instances: Vec::new(),
        stickers: Vec::new(),
        system_channel_flags: SystemChannelFlags::empty(),
        system_channel_id: None,
        threads: Vec::new(),
        unavailable: false,
        vanity_url_code: None,
        verification_level: VerificationLevel::VeryHigh,
        voice_states: Vec::new(),
        widget_channel_id: None,
        widget_enabled: None,
    }
}
