use crate::{
    client::Client,
    error::Error as HttpError,
    request::{Request, TryIntoRequest},
    response::{marker::MemberListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{marker::GuildMarker, Id};
use twilight_validate::request::{
    search_guild_members_limit as validate_search_guild_members_limit, ValidationError,
};

struct SearchGuildMembersFields<'a> {
    query: &'a str,
    limit: Option<u16>,
}

/// Search the members of a specific guild by a query.
///
/// The upper limit to this request is 1000. Discord defaults the limit to 1.
///
/// # Examples
///
/// Get the first 10 members of guild `100` matching `Wumpus`:
///
/// ```no_run
/// use twilight_http::Client;
/// use twilight_model::id::Id;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let guild_id = Id::new(100);
/// let members = client
///     .search_guild_members(guild_id, "Wumpus")
///     .limit(10)?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
///
/// # Errors
///
/// Returns an error of type [`SearchGuildMembers`] if the limit is 0 or greater
/// than 1000.
///
/// [`SearchGuildMembers`]: twilight_validate::request::ValidationErrorType::SearchGuildMembers
#[must_use = "requests must be configured and executed"]
pub struct SearchGuildMembers<'a> {
    fields: SearchGuildMembersFields<'a>,
    guild_id: Id<GuildMarker>,
    http: &'a Client,
}

impl<'a> SearchGuildMembers<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: Id<GuildMarker>, query: &'a str) -> Self {
        Self {
            fields: SearchGuildMembersFields { query, limit: None },
            guild_id,
            http,
        }
    }

    /// Sets the number of members to retrieve per request.
    ///
    /// The limit must be greater than 0 and less than 1000.
    ///
    /// # Errors
    ///
    /// Returns an error of type [`SearchGuildMembers`] if the limit is 0 or
    /// greater than 1000.
    ///
    /// [`SearchGuildMembers`]: twilight_validate::request::ValidationErrorType::SearchGuildMembers
    pub const fn limit(mut self, limit: u16) -> Result<Self, ValidationError> {
        if let Err(source) = validate_search_guild_members_limit(limit) {
            return Err(source);
        }

        self.fields.limit = Some(limit);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<MemberListBody> {
        let guild_id = self.guild_id;
        let http = self.http;

        match self.try_into_request() {
            Ok(request) => {
                let mut future = http.request(request);
                future.set_guild_id(guild_id);

                future
            }
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl TryIntoRequest for SearchGuildMembers<'_> {
    fn try_into_request(self) -> Result<Request, HttpError> {
        Ok(Request::from_route(&Route::SearchGuildMembers {
            guild_id: self.guild_id.get(),
            limit: self.fields.limit,
            query: self.fields.query,
        }))
    }
}
