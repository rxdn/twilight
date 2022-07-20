use crate::{
    client::Client,
    error::Error as HttpError,
    request::{Request, TryIntoRequest},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    id::{marker::GuildMarker, Id},
    template::Template,
};
use twilight_validate::request::{
    template_description as validate_template_description, template_name as validate_template_name,
    ValidationError,
};

#[derive(Serialize)]
struct CreateTemplateFields<'a> {
    name: &'a str,
    description: Option<&'a str>,
}

/// Create a template from the current state of the guild.
///
/// Requires the `MANAGE_GUILD` permission. The name must be at least 1 and at
/// most 100 characters in length.
///
/// # Errors
///
/// Returns an error of type [`TemplateName`] if the name length is too short or
/// too long.
///
/// [`TemplateName`]: twilight_validate::request::ValidationErrorType::TemplateName
#[must_use = "requests must be configured and executed"]
pub struct CreateTemplate<'a> {
    fields: CreateTemplateFields<'a>,
    guild_id: Id<GuildMarker>,
    http: &'a Client,
}

impl<'a> CreateTemplate<'a> {
    pub(crate) fn new(
        http: &'a Client,
        guild_id: Id<GuildMarker>,
        name: &'a str,
    ) -> Result<Self, ValidationError> {
        validate_template_name(name)?;

        Ok(Self {
            fields: CreateTemplateFields {
                name,
                description: None,
            },
            guild_id,
            http,
        })
    }

    /// Set the template's description.
    ///
    /// This must be less than or equal to 120 characters in length.
    ///
    /// # Errors
    ///
    /// Returns an error of type [`TemplateDescription`] if the name length is
    /// too short or too long.
    ///
    /// [`TemplateDescription`]: twilight_validate::request::ValidationErrorType::TemplateDescription
    pub fn description(mut self, description: &'a str) -> Result<Self, ValidationError> {
        validate_template_description(description)?;

        self.fields.description.replace(description);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Template> {
        let http = self.http;

        match self.try_into_request() {
            Ok(request) => http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl TryIntoRequest for CreateTemplate<'_> {
    fn try_into_request(self) -> Result<Request, HttpError> {
        let mut request = Request::builder(&Route::CreateTemplate {
            guild_id: self.guild_id.get(),
        });

        request = request.json(&self.fields)?;

        Ok(request.build())
    }
}
