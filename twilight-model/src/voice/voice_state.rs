use crate::{
    guild::member::{Member, MemberIntermediary},
    id::{
        marker::{ChannelMarker, GuildMarker, UserMarker},
        Id,
    },
    util::Timestamp,
};
use serde::{
    de::{Deserializer, Error as DeError, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::fmt::{Formatter, Result as FmtResult};

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct VoiceState {
    pub channel_id: Option<Id<ChannelMarker>>,
    pub deaf: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Id<GuildMarker>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Member>,
    pub mute: bool,
    pub self_deaf: bool,
    pub self_mute: bool,
    /// Whether this user is streaming via "Go Live".
    #[serde(default)]
    pub self_stream: bool,
    /// Whether this user's camera is enabled.
    pub self_video: bool,
    pub session_id: String,
    pub suppress: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub user_id: Id<UserMarker>,
    /// When the user requested to speak.
    ///
    /// # serde
    ///
    /// This is serialized as an ISO 8601 timestamp in the format of
    /// "2021-01-01T01-01-01.010000+00:00".
    pub request_to_speak_timestamp: Option<Timestamp>,
}

#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Field {
    ChannelId,
    Deaf,
    GuildId,
    Member,
    Mute,
    SelfDeaf,
    SelfMute,
    SelfStream,
    SelfVideo,
    SessionId,
    Suppress,
    Token,
    UserId,
    RequestToSpeakTimestamp,
}

struct VoiceStateVisitor;

impl<'de> Visitor<'de> for VoiceStateVisitor {
    type Value = VoiceState;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("struct VoiceState")
    }

    #[allow(clippy::too_many_lines)]
    fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
        let mut channel_id = None;
        let mut deaf = None;
        let mut guild_id = None;
        let mut member: Option<MemberIntermediary> = None;
        let mut mute = None;
        let mut self_deaf = None;
        let mut self_mute = None;
        let mut self_stream = None;
        let mut self_video = None;
        let mut session_id = None;
        let mut suppress = None;
        let mut token = None;
        let mut user_id = None;
        let mut request_to_speak_timestamp = None;

        let span = tracing::trace_span!("deserializing voice state");
        let _span_enter = span.enter();

        loop {
            let span_child = tracing::trace_span!("iterating over element");
            let _span_child_enter = span_child.enter();

            let key = match map.next_key() {
                Ok(Some(key)) => {
                    tracing::trace!(?key, "found key");

                    key
                }
                Ok(None) => break,
                Err(why) => {
                    // Encountered when we run into an unknown key.
                    map.next_value::<IgnoredAny>()?;

                    tracing::trace!("ran into an unknown key: {why:?}");

                    continue;
                }
            };

            match key {
                Field::ChannelId => {
                    if channel_id.is_some() {
                        return Err(DeError::duplicate_field("channel_id"));
                    }

                    channel_id = map.next_value()?;
                }
                Field::Deaf => {
                    if deaf.is_some() {
                        return Err(DeError::duplicate_field("deaf"));
                    }

                    deaf = Some(map.next_value()?);
                }
                Field::GuildId => {
                    if guild_id.is_some() {
                        return Err(DeError::duplicate_field("guild_id"));
                    }

                    guild_id = map.next_value()?;
                }
                Field::Member => {
                    if member.is_some() {
                        return Err(DeError::duplicate_field("member"));
                    }

                    member = map.next_value()?;
                }
                Field::Mute => {
                    if mute.is_some() {
                        return Err(DeError::duplicate_field("mute"));
                    }

                    mute = Some(map.next_value()?);
                }
                Field::SelfDeaf => {
                    if self_deaf.is_some() {
                        return Err(DeError::duplicate_field("self_deaf"));
                    }

                    self_deaf = Some(map.next_value()?);
                }
                Field::SelfMute => {
                    if self_mute.is_some() {
                        return Err(DeError::duplicate_field("self_mute"));
                    }

                    self_mute = Some(map.next_value()?);
                }
                Field::SelfStream => {
                    if self_stream.is_some() {
                        return Err(DeError::duplicate_field("self_stream"));
                    }

                    self_stream = Some(map.next_value()?);
                }
                Field::SelfVideo => {
                    if self_video.is_some() {
                        return Err(DeError::duplicate_field("self_video"));
                    }

                    self_video = Some(map.next_value()?);
                }
                Field::SessionId => {
                    if session_id.is_some() {
                        return Err(DeError::duplicate_field("session_id"));
                    }

                    session_id = Some(map.next_value()?);
                }
                Field::Suppress => {
                    if suppress.is_some() {
                        return Err(DeError::duplicate_field("suppress"));
                    }

                    suppress = Some(map.next_value()?);
                }
                Field::Token => {
                    if token.is_some() {
                        return Err(DeError::duplicate_field("token"));
                    }

                    token = map.next_value()?;
                }
                Field::UserId => {
                    if user_id.is_some() {
                        return Err(DeError::duplicate_field("user_id"));
                    }

                    user_id = Some(map.next_value()?);
                }
                Field::RequestToSpeakTimestamp => {
                    if request_to_speak_timestamp.is_some() {
                        return Err(DeError::duplicate_field("request_to_speak_timestamp"));
                    }

                    request_to_speak_timestamp = map.next_value()?;
                }
            }
        }

        let deaf = deaf.ok_or_else(|| DeError::missing_field("deaf"))?;
        let mute = mute.ok_or_else(|| DeError::missing_field("mute"))?;
        let self_deaf = self_deaf.ok_or_else(|| DeError::missing_field("self_deaf"))?;
        let self_mute = self_mute.ok_or_else(|| DeError::missing_field("self_mute"))?;
        let self_video = self_video.ok_or_else(|| DeError::missing_field("self_video"))?;
        let session_id = session_id.ok_or_else(|| DeError::missing_field("session_id"))?;
        let suppress = suppress.ok_or_else(|| DeError::missing_field("suppress"))?;
        let user_id = user_id.ok_or_else(|| DeError::missing_field("user_id"))?;

        let self_stream = self_stream.unwrap_or_default();

        tracing::trace!(
            %deaf,
            %mute,
            %self_deaf,
            %self_mute,
            %self_stream,
            %self_video,
            ?session_id,
            %suppress,
            %user_id,
        );

        let member = if let (Some(guild_id), Some(member)) = (guild_id, member) {
            tracing::trace!(%guild_id, ?member, "setting member guild id");

            Some(member.into_member(guild_id))
        } else {
            None
        };

        Ok(VoiceState {
            channel_id,
            deaf,
            guild_id,
            member,
            mute,
            self_deaf,
            self_mute,
            self_stream,
            self_video,
            session_id,
            suppress,
            token,
            user_id,
            request_to_speak_timestamp,
        })
    }
}

impl<'de> Deserialize<'de> for VoiceState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        const FIELDS: &[&str] = &[
            "channel_id",
            "deaf",
            "guild_id",
            "member",
            "mute",
            "self_deaf",
            "self_mute",
            "self_stream",
            "session_id",
            "suppress",
            "token",
            "user_id",
            "request_to_speak_timestamp",
        ];

        deserializer.deserialize_struct("VoiceState", FIELDS, VoiceStateVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{Member, VoiceState};
    use crate::{
        id::Id,
        user::User,
        util::datetime::{Timestamp, TimestampParseError},
    };
    use serde_test::Token;
    use std::str::FromStr;

    #[test]
    fn voice_state() {
        let value = VoiceState {
            channel_id: Some(Id::new(1)),
            deaf: false,
            guild_id: Some(Id::new(2)),
            member: None,
            mute: true,
            self_deaf: false,
            self_mute: true,
            self_stream: false,
            self_video: false,
            session_id: "a".to_owned(),
            suppress: true,
            token: None,
            user_id: Id::new(3),
            request_to_speak_timestamp: None,
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "VoiceState",
                    len: 12,
                },
                Token::Str("channel_id"),
                Token::Some,
                Token::NewtypeStruct { name: "Id" },
                Token::Str("1"),
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("guild_id"),
                Token::Some,
                Token::NewtypeStruct { name: "Id" },
                Token::Str("2"),
                Token::Str("mute"),
                Token::Bool(true),
                Token::Str("self_deaf"),
                Token::Bool(false),
                Token::Str("self_mute"),
                Token::Bool(true),
                Token::Str("self_stream"),
                Token::Bool(false),
                Token::Str("self_video"),
                Token::Bool(false),
                Token::Str("session_id"),
                Token::Str("a"),
                Token::Str("suppress"),
                Token::Bool(true),
                Token::Str("user_id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("3"),
                Token::Str("request_to_speak_timestamp"),
                Token::None,
                Token::StructEnd,
            ],
        );
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn voice_state_complete() -> Result<(), TimestampParseError> {
        let joined_at = Timestamp::from_str("2015-04-26T06:26:56.936000+00:00")?;
        let premium_since = Timestamp::from_str("2021-03-16T14:29:19.046000+00:00")?;
        let request_to_speak_timestamp = Timestamp::from_str("2021-04-21T22:16:50.000000+00:00")?;

        let value = VoiceState {
            channel_id: Some(Id::new(1)),
            deaf: false,
            guild_id: Some(Id::new(2)),
            member: Some(Member {
                avatar: None,
                communication_disabled_until: None,
                deaf: false,
                guild_id: Id::new(2),
                joined_at,
                mute: true,
                nick: Some("twilight".to_owned()),
                pending: false,
                premium_since: Some(premium_since),
                roles: Vec::new(),
                user: User {
                    accent_color: None,
                    avatar: None,
                    banner: None,
                    bot: false,
                    discriminator: 1,
                    email: None,
                    flags: None,
                    id: Id::new(3),
                    locale: None,
                    mfa_enabled: None,
                    name: "twilight".to_owned(),
                    premium_type: None,
                    public_flags: None,
                    system: None,
                    verified: None,
                },
            }),
            mute: true,
            self_deaf: false,
            self_mute: true,
            self_stream: false,
            self_video: false,
            session_id: "a".to_owned(),
            suppress: true,
            token: Some("abc".to_owned()),
            user_id: Id::new(3),
            request_to_speak_timestamp: Some(request_to_speak_timestamp),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "VoiceState",
                    len: 14,
                },
                Token::Str("channel_id"),
                Token::Some,
                Token::NewtypeStruct { name: "Id" },
                Token::Str("1"),
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("guild_id"),
                Token::Some,
                Token::NewtypeStruct { name: "Id" },
                Token::Str("2"),
                Token::Str("member"),
                Token::Some,
                Token::Struct {
                    name: "Member",
                    len: 10,
                },
                Token::Str("communication_disabled_until"),
                Token::None,
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("guild_id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("2"),
                Token::Str("joined_at"),
                Token::Str("2015-04-26T06:26:56.936000+00:00"),
                Token::Str("mute"),
                Token::Bool(true),
                Token::Str("nick"),
                Token::Some,
                Token::Str("twilight"),
                Token::Str("pending"),
                Token::Bool(false),
                Token::Str("premium_since"),
                Token::Some,
                Token::Str("2021-03-16T14:29:19.046000+00:00"),
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
                Token::Str("0001"),
                Token::Str("id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("3"),
                Token::Str("username"),
                Token::Str("twilight"),
                Token::StructEnd,
                Token::StructEnd,
                Token::Str("mute"),
                Token::Bool(true),
                Token::Str("self_deaf"),
                Token::Bool(false),
                Token::Str("self_mute"),
                Token::Bool(true),
                Token::Str("self_stream"),
                Token::Bool(false),
                Token::Str("self_video"),
                Token::Bool(false),
                Token::Str("session_id"),
                Token::Str("a"),
                Token::Str("suppress"),
                Token::Bool(true),
                Token::Str("token"),
                Token::Some,
                Token::Str("abc"),
                Token::Str("user_id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("3"),
                Token::Str("request_to_speak_timestamp"),
                Token::Some,
                Token::Str("2021-04-21T22:16:50.000000+00:00"),
                Token::StructEnd,
            ],
        );

        Ok(())
    }
}
