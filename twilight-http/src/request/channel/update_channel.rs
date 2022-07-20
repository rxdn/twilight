use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, AuditLogReason, Nullable, Request, TryIntoRequest},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, Channel, ChannelType, VideoQualityMode},
    id::{marker::ChannelMarker, Id},
};
use twilight_validate::{
    channel::{name as validate_name, topic as validate_topic, ChannelValidationError},
    request::{audit_reason as validate_audit_reason, ValidationError},
};

// The Discord API doesn't require the `name` and `kind` fields to be present,
// but it does require them to be non-null.
#[derive(Serialize)]
struct UpdateChannelFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    bitrate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<Nullable<Id<ChannelMarker>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permission_overwrites: Option<&'a [PermissionOverwrite]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_limit: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_quality_mode: Option<VideoQualityMode>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<ChannelType>,
}

/// Update a channel.
///
/// All fields are optional. The minimum length of the name is 1 UTF-16 character
/// and the maximum is 100 UTF-16 characters.
#[must_use = "requests must be configured and executed"]
pub struct UpdateChannel<'a> {
    channel_id: Id<ChannelMarker>,
    fields: UpdateChannelFields<'a>,
    http: &'a Client,
    reason: Option<&'a str>,
}

impl<'a> UpdateChannel<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: Id<ChannelMarker>) -> Self {
        Self {
            channel_id,
            fields: UpdateChannelFields {
                bitrate: None,
                name: None,
                nsfw: None,
                parent_id: None,
                permission_overwrites: None,
                position: None,
                rate_limit_per_user: None,
                topic: None,
                user_limit: None,
                video_quality_mode: None,
                kind: None,
            },
            http,
            reason: None,
        }
    }

    /// Set the bitrate of the channel. Applicable to voice channels only.
    pub const fn bitrate(mut self, bitrate: u32) -> Self {
        self.fields.bitrate = Some(bitrate);

        self
    }

    /// Set the name.
    ///
    /// The minimum length is 1 UTF-16 character and the maximum is 100 UTF-16
    /// characters.
    ///
    /// # Errors
    ///
    /// Returns an error of type [`NameInvalid`] if the name is invalid.
    ///
    /// [`NameInvalid`]: twilight_validate::channel::ChannelValidationErrorType::NameInvalid
    pub fn name(mut self, name: &'a str) -> Result<Self, ChannelValidationError> {
        validate_name(name)?;

        self.fields.name = Some(name);

        Ok(self)
    }

    /// Set whether the channel is marked as NSFW.
    pub const fn nsfw(mut self, nsfw: bool) -> Self {
        self.fields.nsfw = Some(nsfw);

        self
    }

    /// If this is specified, and the parent ID is a `ChannelType::CategoryChannel`, move this
    /// channel to a child of the category channel.
    pub const fn parent_id(mut self, parent_id: Option<Id<ChannelMarker>>) -> Self {
        self.fields.parent_id = Some(Nullable(parent_id));

        self
    }

    /// Set the permission overwrites of a channel. This will overwrite all permissions that the
    /// channel currently has, so use with caution!
    pub const fn permission_overwrites(
        mut self,
        permission_overwrites: &'a [PermissionOverwrite],
    ) -> Self {
        self.fields.permission_overwrites = Some(permission_overwrites);

        self
    }

    /// Set the position of the channel.
    ///
    /// Positions are numerical and zero-indexed. If you place a channel at position 2, channels
    /// 2-n will shift down one position and the initial channel will take its place.
    pub const fn position(mut self, position: u64) -> Self {
        self.fields.position = Some(position);

        self
    }

    /// Set the number of seconds that a user must wait before before they are able to send another
    /// message.
    ///
    /// The minimum is 0 and the maximum is 21600. This is also known as "Slow
    /// Mode". See [Discord Docs/Channel Object].
    ///
    /// # Errors
    ///
    /// Returns an error of type [`RateLimitPerUserInvalid`] if the name is
    /// invalid.
    ///
    /// [`RateLimitPerUserInvalid`]: twilight_validate::channel::ChannelValidationErrorType::RateLimitPerUserInvalid
    /// [Discord Docs/Channel Object]: https://discordapp.com/developers/docs/resources/channel#channel-object-channel-structure
    pub const fn rate_limit_per_user(
        mut self,
        rate_limit_per_user: u16,
    ) -> Result<Self, ChannelValidationError> {
        if let Err(source) = twilight_validate::channel::rate_limit_per_user(rate_limit_per_user) {
            return Err(source);
        }

        self.fields.rate_limit_per_user = Some(rate_limit_per_user);

        Ok(self)
    }

    /// Set the topic.
    ///
    /// The maximum length is 1024 UTF-16 characters. See
    /// [Discord Docs/Channel Object].
    ///
    /// # Errors
    ///
    /// Returns an error of type [`TopicInvalid`] if the name is
    /// invalid.
    ///
    /// [Discord Docs/Channel Object]: https://discordapp.com/developers/docs/resources/channel#channel-object-channel-structure
    /// [`TopicInvalid`]: twilight_validate::channel::ChannelValidationErrorType::TopicInvalid
    pub fn topic(mut self, topic: &'a str) -> Result<Self, ChannelValidationError> {
        validate_topic(topic)?;

        self.fields.topic.replace(topic);

        Ok(self)
    }

    /// For voice channels, set the user limit.
    ///
    /// Set to 0 for no limit. Limit can otherwise be between 1 and 99
    /// inclusive. See [Discord Docs/Modify Channel].
    ///
    /// [Discord Docs/Modify Channel]: https://discord.com/developers/docs/resources/channel#modify-channel-json-params-guild-channel
    pub const fn user_limit(mut self, user_limit: u16) -> Self {
        self.fields.user_limit = Some(user_limit);

        self
    }

    /// Set the [`VideoQualityMode`] for the voice channel.
    pub const fn video_quality_mode(mut self, video_quality_mode: VideoQualityMode) -> Self {
        self.fields.video_quality_mode = Some(video_quality_mode);

        self
    }

    /// Set the kind of channel.
    ///
    /// Only conversion between `ChannelType::GuildText` and
    /// `ChannelType::GuildNews` is possible, and only if the guild has the
    /// `NEWS` feature enabled. See [Discord Docs/Modify Channel].
    ///
    /// [Discord Docs/Modify Channel]: https://discord.com/developers/docs/resources/channel#modify-channel-json-params-guild-channel
    pub const fn kind(mut self, kind: ChannelType) -> Self {
        self.fields.kind = Some(kind);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Channel> {
        let http = self.http;

        match self.try_into_request() {
            Ok(request) => http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl<'a> AuditLogReason<'a> for UpdateChannel<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, ValidationError> {
        validate_audit_reason(reason)?;

        self.reason.replace(reason);

        Ok(self)
    }
}

impl TryIntoRequest for UpdateChannel<'_> {
    fn try_into_request(self) -> Result<Request, HttpError> {
        let mut request = Request::builder(&Route::UpdateChannel {
            channel_id: self.channel_id.get(),
        })
        .json(&self.fields)?;

        if let Some(reason) = &self.reason {
            request = request.headers(request::audit_header(reason)?);
        }

        Ok(request.build())
    }
}
