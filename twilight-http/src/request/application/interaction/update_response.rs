//! Update a original response create for a interaction.

use crate::{
    client::Client,
    error::Error as HttpError,
    request::{
        attachment::{AttachmentManager, PartialAttachment},
        Nullable, Request, TryIntoRequest,
    },
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    application::component::Component,
    channel::{embed::Embed, message::AllowedMentions, Message},
    http::attachment::Attachment,
    id::{
        marker::{ApplicationMarker, AttachmentMarker},
        Id,
    },
};
use twilight_validate::message::{
    attachment_filename as validate_attachment_filename, components as validate_components,
    content as validate_content, embeds as validate_embeds, MessageValidationError,
};

#[derive(Serialize)]
struct UpdateResponseFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<Nullable<&'a AllowedMentions>>,
    /// List of attachments to keep, and new attachments to add.
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Nullable<Vec<PartialAttachment<'a>>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Nullable<&'a [Component]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Nullable<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Nullable<&'a [Embed]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_json: Option<&'a [u8]>,
}

/// Edit the original message, by its token.
///
/// You can pass [`None`] to any of the methods to remove the associated field.
/// Pass [`None`] to [`content`] to remove the content. You must ensure that the
/// message still contains at least one of [`attachments`], [`content`], or
/// [`embeds`].
///
/// # Examples
///
/// Update the original response by setting the content to `test <@3>` -
/// attempting to mention user ID 3 - while specifying that no entities can be
/// mentioned.
///
/// ```no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::env;
/// use twilight_http::Client;
/// use twilight_model::{channel::message::AllowedMentions, id::Id};
///
/// let client = Client::new(env::var("DISCORD_TOKEN")?);
/// let application_id = Id::new(1);
///
/// client
///     .interaction(application_id)
///     .update_response("token here")
///     // By creating a default set of allowed mentions, no entity can be
///     // mentioned.
///     .allowed_mentions(Some(&AllowedMentions::default()))
///     .content(Some("test <@3>"))?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`attachments`]: Self::attachments
/// [`content`]: Self::content
/// [`embeds`]: Self::embeds
#[must_use = "requests must be configured and executed"]
pub struct UpdateResponse<'a> {
    application_id: Id<ApplicationMarker>,
    attachment_manager: AttachmentManager<'a>,
    fields: UpdateResponseFields<'a>,
    http: &'a Client,
    token: &'a str,
}

impl<'a> UpdateResponse<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        application_id: Id<ApplicationMarker>,
        interaction_token: &'a str,
    ) -> Self {
        Self {
            application_id,
            attachment_manager: AttachmentManager::new(),
            fields: UpdateResponseFields {
                allowed_mentions: None,
                attachments: None,
                components: None,
                content: None,
                embeds: None,
                payload_json: None,
            },
            http,
            token: interaction_token,
        }
    }

    /// Specify the [`AllowedMentions`] for the message.
    ///
    /// If not called, the request will use the client's default allowed
    /// mentions.
    pub const fn allowed_mentions(mut self, allowed_mentions: Option<&'a AllowedMentions>) -> Self {
        self.fields.allowed_mentions = Some(Nullable(allowed_mentions));

        self
    }

    /// Attach multiple new files to the message.
    ///
    /// This method clears previous calls.
    ///
    /// # Errors
    ///
    /// Returns an error of type [`AttachmentFilename`] if any filename is
    /// invalid.
    ///
    /// [`AttachmentFilename`]: twilight_validate::message::MessageValidationErrorType::AttachmentFilename
    pub fn attachments(
        mut self,
        attachments: &'a [Attachment],
    ) -> Result<Self, MessageValidationError> {
        attachments
            .iter()
            .try_for_each(|attachment| validate_attachment_filename(&attachment.filename))?;

        self.attachment_manager = self
            .attachment_manager
            .set_files(attachments.iter().collect());

        Ok(self)
    }

    /// Set the message's list of [`Component`]s.
    ///
    /// Calling this method will clear previous calls.
    ///
    /// # Editing
    ///
    /// Pass [`None`] to clear existing components.
    ///
    /// # Errors
    ///
    /// Refer to the errors section of
    /// [`twilight_validate::component::component`] for a list of errors that
    /// may be returned as a result of validating each provided component.
    pub fn components(
        mut self,
        components: Option<&'a [Component]>,
    ) -> Result<Self, MessageValidationError> {
        if let Some(components) = components {
            validate_components(components)?;
        }

        self.fields.components = Some(Nullable(components));

        Ok(self)
    }

    /// Set the message's content.
    ///
    /// The maximum length is 2000 UTF-16 characters.
    ///
    /// # Editing
    ///
    /// Pass [`None`] to remove the message content. This is impossible if it
    /// would leave the message empty of `attachments`, `content`, or `embeds`.
    ///
    /// # Errors
    ///
    /// Returns an error of type [`ContentInvalid`] if the content length is too
    /// long.
    ///
    /// [`ContentInvalid`]: twilight_validate::message::MessageValidationErrorType::ContentInvalid
    pub fn content(mut self, content: Option<&'a str>) -> Result<Self, MessageValidationError> {
        if let Some(content_ref) = content {
            validate_content(content_ref)?;
        }

        self.fields.content = Some(Nullable(content));

        Ok(self)
    }

    /// Set the message's list of embeds.
    ///
    /// Calling this method will clear previous calls.
    ///
    /// The amount of embeds must not exceed [`EMBED_COUNT_LIMIT`]. The total
    /// character length of each embed must not exceed [`EMBED_TOTAL_LENGTH`]
    /// characters. Additionally, the internal fields also have character
    /// limits. See [Discord Docs/Embed Limits].
    ///
    /// # Editing
    ///
    /// To keep all embeds, do not call this method. To modify one or more
    /// embeds in the message, acquire them from the previous message, mutate
    /// them in place, then pass that list to this method. To remove all embeds,
    /// pass [`None`]. This is impossible if it would leave the message empty of
    /// `attachments`, `content`, or `embeds`.
    ///
    /// # Examples
    ///
    /// Create an embed and update the message with the new embed. The content
    /// of the original message is unaffected and only the embed(s) are
    /// modified.
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use twilight_http::Client;
    /// use twilight_model::id::Id;
    /// use twilight_util::builder::embed::EmbedBuilder;
    ///
    /// let client = Client::new("token".to_owned());
    /// let application_id = Id::new(1);
    ///
    /// let embed = EmbedBuilder::new()
    ///     .description(
    ///         "Powerful, flexible, and scalable ecosystem of Rust \
    ///     libraries for the Discord API.",
    ///     )
    ///     .title("Twilight")
    ///     .url("https://twilight.rs")
    ///     .validate()?
    ///     .build();
    ///
    /// client
    ///     .interaction(application_id)
    ///     .update_response("token")
    ///     .embeds(Some(&[embed]))?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error of type [`TooManyEmbeds`] if there are too many embeds.
    ///
    /// Otherwise, refer to the errors section of
    /// [`twilight_validate::embed::embed`] for a list of errors that may occur.
    ///
    /// [`EMBED_COUNT_LIMIT`]: twilight_validate::message::EMBED_COUNT_LIMIT
    /// [`EMBED_TOTAL_LENGTH`]: twilight_validate::embed::EMBED_TOTAL_LENGTH
    /// [`TooManyEmbeds`]: twilight_validate::message::MessageValidationErrorType::TooManyEmbeds
    /// [Discord Docs/Embed Limits]: https://discord.com/developers/docs/resources/channel#embed-limits
    pub fn embeds(mut self, embeds: Option<&'a [Embed]>) -> Result<Self, MessageValidationError> {
        if let Some(embeds) = embeds {
            validate_embeds(embeds)?;
        }

        self.fields.embeds = Some(Nullable(embeds));

        Ok(self)
    }

    /// Specify multiple [`Id<AttachmentMarker>`]s already present in the target
    /// message to keep.
    ///
    /// If called, all unspecified attachments (except ones added with
    /// [`attachments`]) will be removed from the message. This is impossible if
    /// it would leave the message empty of `attachments`, `content`, or
    /// `embeds`. If not called, all attachments will be kept.
    ///
    /// [`attachments`]: Self::attachments
    pub fn keep_attachment_ids(mut self, attachment_ids: &'a [Id<AttachmentMarker>]) -> Self {
        self.attachment_manager = self.attachment_manager.set_ids(attachment_ids.to_vec());

        // Set an empty list. This will be overwritten in `TryIntoRequest` if
        // the actual list is not empty.
        self.fields.attachments = Some(Nullable(Some(Vec::new())));

        self
    }

    /// JSON encoded body of any additional request fields.
    ///
    /// If this method is called, all other fields are ignored, except for
    /// [`attachments`]. See [Discord Docs/Uploading Files].
    ///
    /// # Examples
    ///
    /// See [`ExecuteWebhook::payload_json`] for examples.
    ///
    /// [Discord Docs/Uploading Files]: https://discord.com/developers/docs/reference#uploading-files
    /// [`ExecuteWebhook::payload_json`]: crate::request::channel::webhook::ExecuteWebhook::payload_json
    /// [`attachments`]: Self::attachments
    pub const fn payload_json(mut self, payload_json: &'a [u8]) -> Self {
        self.fields.payload_json = Some(payload_json);

        self
    }

    pub fn exec(self) -> ResponseFuture<Message> {
        let http = self.http;

        match self.try_into_request() {
            Ok(request) => http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl TryIntoRequest for UpdateResponse<'_> {
    fn try_into_request(mut self) -> Result<Request, HttpError> {
        let mut request = Request::builder(&Route::UpdateInteractionOriginal {
            application_id: self.application_id.get(),
            interaction_token: self.token,
        });

        // Interaction executions don't need the authorization token, only the
        // interaction token.
        request = request.use_authorization_token(false);

        // Set the default allowed mentions if required.
        if self.fields.allowed_mentions.is_none() {
            if let Some(allowed_mentions) = self.http.default_allowed_mentions() {
                self.fields.allowed_mentions = Some(Nullable(Some(allowed_mentions)));
            }
        }

        // Determine whether we need to use a multipart/form-data body or a JSON
        // body.
        if !self.attachment_manager.is_empty() {
            let form = if let Some(payload_json) = self.fields.payload_json {
                self.attachment_manager.build_form(payload_json)
            } else {
                self.fields.attachments = Some(Nullable(Some(
                    self.attachment_manager.get_partial_attachments(),
                )));

                let fields = crate::json::to_vec(&self.fields).map_err(HttpError::json)?;

                self.attachment_manager.build_form(fields.as_ref())
            };

            request = request.form(form);
        } else if let Some(payload_json) = self.fields.payload_json {
            request = request.body(payload_json.to_vec());
        } else {
            request = request.json(&self.fields)?;
        }

        Ok(request.build())
    }
}

#[cfg(test)]
mod tests {
    use crate::{client::Client, request::TryIntoRequest};
    use std::error::Error;
    use twilight_http_ratelimiting::Path;
    use twilight_model::id::Id;

    #[test]
    fn delete_followup_message() -> Result<(), Box<dyn Error>> {
        let application_id = Id::new(1);
        let token = "foo".to_owned();

        let client = Client::new(String::new());
        let req = client
            .interaction(application_id)
            .update_response(&token)
            .content(Some("test"))?
            .try_into_request()?;

        assert!(!req.use_authorization_token());
        assert_eq!(
            &Path::WebhooksIdTokenMessagesId(application_id.get(), token),
            req.ratelimit_path()
        );

        Ok(())
    }
}
