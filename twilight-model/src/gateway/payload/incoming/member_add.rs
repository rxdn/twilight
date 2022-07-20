use crate::guild::Member;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MemberAdd(pub Member);

impl Deref for MemberAdd {
    type Target = Member;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MemberAdd {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Member, MemberAdd};
    use crate::{id::Id, user::User, util::Timestamp};
    use serde_test::Token;

    #[test]
    fn member_add() {
        let joined_at = Timestamp::from_secs(1_632_072_645).expect("non zero");

        let value = MemberAdd(Member {
            avatar: None,
            communication_disabled_until: None,
            deaf: false,
            guild_id: Id::new(1),
            joined_at,
            mute: false,
            nick: None,
            pending: true,
            premium_since: None,
            roles: vec![],
            user: User {
                id: Id::new(2),
                accent_color: None,
                avatar: None,
                banner: None,
                bot: false,
                discriminator: 987,
                name: "ab".to_string(),
                mfa_enabled: None,
                locale: None,
                verified: None,
                email: None,
                flags: None,
                premium_type: None,
                system: None,
                public_flags: None,
            },
        });

        serde_test::assert_tokens(
            &value,
            &[
                Token::NewtypeStruct { name: "MemberAdd" },
                Token::Struct {
                    name: "Member",
                    len: 9,
                },
                Token::Str("communication_disabled_until"),
                Token::None,
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("guild_id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("1"),
                Token::Str("joined_at"),
                Token::Str("2021-09-19T17:30:45.000000+00:00"),
                Token::Str("mute"),
                Token::Bool(false),
                Token::Str("nick"),
                Token::None,
                Token::Str("pending"),
                Token::Bool(true),
                Token::Str("roles"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("user"),
                Token::Struct {
                    name: "User",
                    len: 7,
                },
                Token::Str("accent_color"),
                Token::None,
                Token::Str("avatar"),
                Token::None,
                Token::Str("banner"),
                Token::None,
                Token::Str("bot"),
                Token::Bool(false),
                Token::Str("discriminator"),
                Token::Str("0987"),
                Token::Str("id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("2"),
                Token::Str("username"),
                Token::Str("ab"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
