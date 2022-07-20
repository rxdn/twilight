use crate::{config::ResourceType, model::CachedMessage, InMemoryCache, UpdateCache};
use std::borrow::Cow;
use twilight_model::gateway::payload::incoming::{
    MessageCreate, MessageDelete, MessageDeleteBulk, MessageUpdate,
};

impl UpdateCache for MessageCreate {
    fn update(&self, cache: &InMemoryCache) {
        if cache.wants(ResourceType::USER) {
            cache.cache_user(Cow::Borrowed(&self.author), self.guild_id);
        }

        if let (Some(member), Some(guild_id), true) = (
            &self.member,
            self.guild_id,
            cache.wants(ResourceType::MEMBER),
        ) {
            cache.cache_borrowed_partial_member(guild_id, member, self.author.id);
        }

        if !cache.wants(ResourceType::MESSAGE) {
            return;
        }

        let mut channel_messages = cache.channel_messages.entry(self.0.channel_id).or_default();

        // If the channel has more messages than the cache size the user has
        // requested then we pop a message ID out. Once we have the popped ID we
        // can remove it from the message cache. This prevents the cache from
        // filling up with old messages that aren't in any channel cache.
        if channel_messages.len() >= cache.config.message_cache_size() {
            if let Some(popped_id) = channel_messages.pop_back() {
                cache.messages.remove(&popped_id);
            }
        }

        channel_messages.push_front(self.0.id);
        cache
            .messages
            .insert(self.0.id, CachedMessage::from(self.0.clone()));
    }
}

impl UpdateCache for MessageDelete {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::MESSAGE) {
            return;
        }

        cache.messages.remove(&self.id);

        let mut channel_messages = cache.channel_messages.entry(self.channel_id).or_default();

        if let Some(idx) = channel_messages.iter().position(|id| *id == self.id) {
            channel_messages.remove(idx);
        }
    }
}

impl UpdateCache for MessageDeleteBulk {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::MESSAGE) {
            return;
        }

        let mut channel_messages = cache.channel_messages.entry(self.channel_id).or_default();

        for id in &self.ids {
            cache.messages.remove(id);

            if let Some(idx) = channel_messages
                .iter()
                .position(|message_id| message_id == id)
            {
                channel_messages.remove(idx);
            }
        }
    }
}

impl UpdateCache for MessageUpdate {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::MESSAGE) {
            return;
        }

        if let Some(mut message) = cache.messages.get_mut(&self.id) {
            if let Some(attachments) = &self.attachments {
                message.attachments = attachments.clone();
            }

            if let Some(content) = &self.content {
                message.content = content.clone();
            }

            if let Some(edited_timestamp) = self.edited_timestamp {
                message.edited_timestamp.replace(edited_timestamp);
            }

            if let Some(embeds) = &self.embeds {
                message.embeds = embeds.clone();
            }

            if let Some(mention_everyone) = self.mention_everyone {
                message.mention_everyone = mention_everyone;
            }

            if let Some(mention_roles) = &self.mention_roles {
                message.mention_roles = mention_roles.clone();
            }

            if let Some(mentions) = &self.mentions {
                message.mentions = mentions.iter().map(|x| x.id).collect::<Vec<_>>();
            }

            if let Some(pinned) = self.pinned {
                message.pinned = pinned;
            }

            if let Some(timestamp) = self.timestamp {
                message.timestamp = timestamp;
            }

            if let Some(tts) = self.tts {
                message.tts = tts;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{InMemoryCache, ResourceType};
    use twilight_model::{
        channel::message::{Message, MessageFlags, MessageType},
        gateway::payload::incoming::MessageCreate,
        guild::PartialMember,
        id::Id,
        user::User,
        util::{image_hash::ImageHashParseError, ImageHash, Timestamp},
    };

    #[test]
    fn message_create() -> Result<(), ImageHashParseError> {
        let joined_at = Timestamp::from_secs(1_632_072_645).expect("non zero");
        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::MESSAGE | ResourceType::MEMBER | ResourceType::USER)
            .message_cache_size(2)
            .build();

        let avatar = ImageHash::parse(b"e91c75bc7656063cc745f4e79d0b7664")?;
        let mut msg = Message {
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

        cache.update(&MessageCreate(msg.clone()));
        msg.id = Id::new(5);
        cache.update(&MessageCreate(msg));

        {
            let entry = cache.user_guilds.get(&Id::new(3)).unwrap();
            assert_eq!(entry.value().len(), 1);
        }
        assert_eq!(
            cache.member(Id::new(1), Id::new(3)).unwrap().user_id,
            Id::new(3),
        );
        {
            let entry = cache.channel_messages.get(&Id::new(2)).unwrap();
            assert_eq!(entry.value().len(), 2);
        }

        let messages = cache
            .channel_messages(Id::new(2))
            .expect("channel is in cache");

        let mut iter = messages.iter();
        // messages are iterated over in descending order from insertion
        assert_eq!(Some(&Id::new(5)), iter.next());
        assert_eq!(Some(&Id::new(4)), iter.next());
        assert!(iter.next().is_none());

        Ok(())
    }
}
