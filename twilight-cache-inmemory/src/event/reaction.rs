use crate::{config::ResourceType, InMemoryCache, UpdateCache};
use twilight_model::{
    channel::{message::MessageReaction, ReactionType},
    gateway::payload::incoming::{
        ReactionAdd, ReactionRemove, ReactionRemoveAll, ReactionRemoveEmoji,
    },
};

impl UpdateCache for ReactionAdd {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::REACTION) {
            return;
        }

        let key = self.0.message_id;

        let mut message = if let Some(message) = cache.messages.get_mut(&key) {
            message
        } else {
            return;
        };

        if let Some(reaction) = message
            .reactions
            .iter_mut()
            .find(|r| reactions_eq(&r.emoji, &self.0.emoji))
        {
            if !reaction.me {
                if let Some(current_user) = cache.current_user() {
                    if current_user.id == self.0.user_id {
                        reaction.me = true;
                    }
                }
            }

            reaction.count += 1;
        } else {
            let me = cache
                .current_user()
                .map(|user| user.id == self.0.user_id)
                .unwrap_or_default();

            message.reactions.push(MessageReaction {
                count: 1,
                emoji: self.0.emoji.clone(),
                me,
            });
        }
    }
}

impl UpdateCache for ReactionRemove {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::REACTION) {
            return;
        }

        let mut message = if let Some(message) = cache.messages.get_mut(&self.0.message_id) {
            message
        } else {
            return;
        };

        if let Some(reaction) = message
            .reactions
            .iter_mut()
            .find(|r| reactions_eq(&r.emoji, &self.0.emoji))
        {
            if reaction.me {
                if let Some(current_user) = cache.current_user() {
                    if current_user.id == self.0.user_id {
                        reaction.me = false;
                    }
                }
            }

            if reaction.count > 1 {
                reaction.count -= 1;
            } else {
                message
                    .reactions
                    .retain(|e| !(reactions_eq(&e.emoji, &self.0.emoji)));
            }
        }
    }
}

impl UpdateCache for ReactionRemoveAll {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::REACTION) {
            return;
        }

        let mut message = if let Some(message) = cache.messages.get_mut(&self.message_id) {
            message
        } else {
            return;
        };

        message.reactions.clear();
    }
}

impl UpdateCache for ReactionRemoveEmoji {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::REACTION) {
            return;
        }

        let mut message = if let Some(message) = cache.messages.get_mut(&self.message_id) {
            message
        } else {
            return;
        };

        let maybe_index = message
            .reactions
            .iter()
            .position(|r| reactions_eq(&r.emoji, &self.emoji));

        if let Some(index) = maybe_index {
            message.reactions.remove(index);
        }
    }
}

fn reactions_eq(a: &ReactionType, b: &ReactionType) -> bool {
    match (a, b) {
        (ReactionType::Custom { id: id_a, .. }, ReactionType::Custom { id: id_b, .. }) => {
            id_a == id_b
        }
        (ReactionType::Unicode { name: name_a }, ReactionType::Unicode { name: name_b }) => {
            name_a == name_b
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::{event::reaction::reactions_eq, test, CachedMessage};
    use twilight_model::{
        channel::{message::MessageReaction, Reaction, ReactionType},
        gateway::payload::incoming::{ReactionRemove, ReactionRemoveAll, ReactionRemoveEmoji},
        id::Id,
    };

    fn find_custom_react(msg: &CachedMessage) -> Option<&MessageReaction> {
        msg.reactions.iter().find(|&r| {
            reactions_eq(
                &r.emoji,
                &ReactionType::Custom {
                    animated: false,
                    id: Id::new(6),
                    name: None,
                },
            )
        })
    }

    #[test]
    fn reaction_add() {
        let cache = test::cache_with_message_and_reactions();
        let msg = cache.message(Id::new(4)).unwrap();

        assert_eq!(msg.reactions.len(), 3);

        let world_react = msg
            .reactions
            .iter()
            .find(|&r| matches!(&r.emoji, ReactionType::Unicode {name} if name == "🗺️"));
        let smiley_react = msg
            .reactions
            .iter()
            .find(|&r| matches!(&r.emoji, ReactionType::Unicode {name} if name == "😀"));
        let custom_react = find_custom_react(&msg);

        assert!(world_react.is_some());
        assert_eq!(world_react.unwrap().count, 1);
        assert!(smiley_react.is_some());
        assert_eq!(smiley_react.unwrap().count, 2);
        assert!(custom_react.is_some());
        assert_eq!(custom_react.unwrap().count, 1);
    }

    #[test]
    fn reaction_remove() {
        let cache = test::cache_with_message_and_reactions();
        cache.update(&ReactionRemove(Reaction {
            channel_id: Id::new(2),
            emoji: ReactionType::Unicode {
                name: "😀".to_owned(),
            },
            guild_id: Some(Id::new(1)),
            member: None,
            message_id: Id::new(4),
            user_id: Id::new(5),
        }));
        cache.update(&ReactionRemove(Reaction {
            channel_id: Id::new(2),
            emoji: ReactionType::Custom {
                animated: false,
                id: Id::new(6),
                name: None,
            },
            guild_id: Some(Id::new(1)),
            member: None,
            message_id: Id::new(4),
            user_id: Id::new(5),
        }));

        let msg = cache.message(Id::new(4)).unwrap();

        assert_eq!(msg.reactions.len(), 2);

        let world_react = msg
            .reactions
            .iter()
            .find(|&r| matches!(&r.emoji, ReactionType::Unicode {name} if name == "🗺️"));
        let smiley_react = msg
            .reactions
            .iter()
            .find(|&r| matches!(&r.emoji, ReactionType::Unicode {name} if name == "😀"));
        let custom_react = find_custom_react(&msg);

        assert!(world_react.is_some());
        assert_eq!(world_react.unwrap().count, 1);
        assert!(smiley_react.is_some());
        assert_eq!(smiley_react.unwrap().count, 1);
        assert!(custom_react.is_none());
    }

    #[test]
    fn reaction_remove_all() {
        let cache = test::cache_with_message_and_reactions();
        cache.update(&ReactionRemoveAll {
            channel_id: Id::new(2),
            message_id: Id::new(4),
            guild_id: Some(Id::new(1)),
        });

        let msg = cache.message(Id::new(4)).unwrap();

        assert_eq!(msg.reactions.len(), 0);
    }

    #[test]
    fn reaction_remove_emoji() {
        let cache = test::cache_with_message_and_reactions();
        cache.update(&ReactionRemoveEmoji {
            channel_id: Id::new(2),
            emoji: ReactionType::Unicode {
                name: "😀".to_owned(),
            },
            guild_id: Id::new(1),
            message_id: Id::new(4),
        });
        cache.update(&ReactionRemoveEmoji {
            channel_id: Id::new(2),
            emoji: ReactionType::Custom {
                animated: false,
                id: Id::new(6),
                name: None,
            },
            guild_id: Id::new(1),
            message_id: Id::new(4),
        });

        let msg = cache.message(Id::new(4)).unwrap();

        assert_eq!(msg.reactions.len(), 1);

        let world_react = msg
            .reactions
            .iter()
            .find(|&r| matches!(&r.emoji, ReactionType::Unicode {name} if name == "🗺️"));
        let smiley_react = msg
            .reactions
            .iter()
            .find(|&r| matches!(&r.emoji, ReactionType::Unicode {name} if name == "😀"));
        let custom_react = find_custom_react(&msg);

        assert!(world_react.is_some());
        assert_eq!(world_react.unwrap().count, 1);
        assert!(smiley_react.is_none());
        assert!(custom_react.is_none());
    }
}
