use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, AuditLogReason, Request, TryIntoRequest},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};
use twilight_validate::request::{
    audit_reason as validate_audit_reason,
    create_guild_ban_delete_message_days as validate_create_guild_ban_delete_message_days,
    ValidationError,
};

struct CreateBanFields<'a> {
    delete_message_days: Option<u16>,
    reason: Option<&'a str>,
}

/// Bans a user from a guild, optionally with the number of days' worth of
/// messages to delete and the reason.
///
/// # Examples
///
/// Ban user `200` from guild `100`, deleting
/// 1 day's worth of messages, for the reason `"memes"`:
///
/// ```no_run
/// use twilight_http::{request::AuditLogReason, Client};
/// use twilight_model::id::Id;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let guild_id = Id::new(100);
/// let user_id = Id::new(200);
/// client
///     .create_ban(guild_id, user_id)
///     .delete_message_days(1)?
///     .reason("memes")?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
#[must_use = "requests must be configured and executed"]
pub struct CreateBan<'a> {
    fields: CreateBanFields<'a>,
    guild_id: Id<GuildMarker>,
    http: &'a Client,
    user_id: Id<UserMarker>,
}

impl<'a> CreateBan<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Self {
        Self {
            fields: CreateBanFields {
                delete_message_days: None,
                reason: None,
            },
            guild_id,
            http,
            user_id,
        }
    }

    /// Set the number of days' worth of messages to delete.
    ///
    /// The number of days must be less than or equal to 7.
    ///
    /// # Errors
    ///
    /// Returns an error of type [`CreateGuildBanDeleteMessageDays`] if the
    /// number of days is greater than 7.
    ///
    /// [`CreateGuildBanDeleteMessageDays`]: twilight_validate::request::ValidationErrorType::CreateGuildBanDeleteMessageDays
    pub const fn delete_message_days(mut self, days: u16) -> Result<Self, ValidationError> {
        if let Err(source) = validate_create_guild_ban_delete_message_days(days) {
            return Err(source);
        }

        self.fields.delete_message_days = Some(days);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let http = self.http;

        match self.try_into_request() {
            Ok(request) => http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl<'a> AuditLogReason<'a> for CreateBan<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, ValidationError> {
        validate_audit_reason(reason)?;

        self.fields.reason.replace(reason);

        Ok(self)
    }
}

impl TryIntoRequest for CreateBan<'_> {
    fn try_into_request(self) -> Result<Request, HttpError> {
        let mut request = Request::builder(&Route::CreateBan {
            delete_message_days: self.fields.delete_message_days,
            guild_id: self.guild_id.get(),
            user_id: self.user_id.get(),
        });

        if let Some(reason) = self.fields.reason.as_ref() {
            let header = request::audit_header(reason)?;

            request = request.headers(header);
        }

        Ok(request.build())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        client::Client,
        request::{AuditLogReason, TryIntoRequest, REASON_HEADER_NAME},
    };
    use hyper::header::HeaderValue;
    use std::error::Error;
    use twilight_http_ratelimiting::Method;
    use twilight_model::id::{
        marker::{GuildMarker, UserMarker},
        Id,
    };

    #[test]
    fn request() -> Result<(), Box<dyn Error>> {
        const GUILD_ID: Id<GuildMarker> = Id::new(1);
        const REASON: &str = "spam";
        const USER_ID: Id<UserMarker> = Id::new(2);

        let client = Client::new(String::new());
        let request = client
            .create_ban(GUILD_ID, USER_ID)
            .reason(REASON)?
            .try_into_request()?;

        assert!(request.body().is_none());
        assert!(request.form().is_none());
        assert_eq!(Method::Put, request.method());

        let header = HeaderValue::from_static(REASON);
        assert!(matches!(
            request.headers(),
            Some(map)
            if map.len() == 1 && map.get(REASON_HEADER_NAME) == Some(&header)));
        assert!(request.use_authorization_token());

        Ok(())
    }
}
