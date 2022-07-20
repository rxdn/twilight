use crate::{
    client::Client,
    error::Error as HttpError,
    request::{Request, TryIntoRequest},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::{stage_instance::PrivacyLevel, StageInstance},
    id::{marker::ChannelMarker, Id},
};
use twilight_validate::request::{stage_topic as validate_stage_topic, ValidationError};

#[derive(Serialize)]
struct CreateStageInstanceFields<'a> {
    channel_id: Id<ChannelMarker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    privacy_level: Option<PrivacyLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    send_start_notification: Option<bool>,
    topic: &'a str,
}

/// Create a new stage instance associated with a stage channel.
///
/// Requires the user to be a moderator of the stage channel.
#[must_use = "requests must be configured and executed"]
pub struct CreateStageInstance<'a> {
    fields: CreateStageInstanceFields<'a>,
    http: &'a Client,
}

impl<'a> CreateStageInstance<'a> {
    pub(crate) fn new(
        http: &'a Client,
        channel_id: Id<ChannelMarker>,
        topic: &'a str,
    ) -> Result<Self, ValidationError> {
        validate_stage_topic(topic)?;

        Ok(Self {
            fields: CreateStageInstanceFields {
                channel_id,
                privacy_level: None,
                send_start_notification: None,
                topic,
            },
            http,
        })
    }

    /// Set the [`PrivacyLevel`] of the instance.
    pub const fn privacy_level(mut self, privacy_level: PrivacyLevel) -> Self {
        self.fields.privacy_level = Some(privacy_level);

        self
    }

    /// Set whether to notify everyone when a stage starts.
    ///
    /// The stage moderator must have [`Permissions::MENTION_EVERYONE`] for this
    /// notification to be sent.
    ///
    /// [`Permissions::MENTION_EVERYONE`]: twilight_model::guild::Permissions::MENTION_EVERYONE
    pub const fn send_start_notification(mut self, send_start_notification: bool) -> Self {
        self.fields.send_start_notification = Some(send_start_notification);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<StageInstance> {
        let http = self.http;

        match self.try_into_request() {
            Ok(request) => http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl TryIntoRequest for CreateStageInstance<'_> {
    fn try_into_request(self) -> Result<Request, HttpError> {
        let mut request = Request::builder(&Route::CreateStageInstance);

        request = request.json(&self.fields)?;

        Ok(request.build())
    }
}
