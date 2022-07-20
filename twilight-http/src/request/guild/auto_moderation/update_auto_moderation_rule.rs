use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, AuditLogReason, Request, TryIntoRequest},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    guild::auto_moderation::{
        AutoModerationAction, AutoModerationEventType, AutoModerationRule,
        AutoModerationTriggerMetadata,
    },
    id::{
        marker::{AutoModerationRuleMarker, ChannelMarker, GuildMarker, RoleMarker},
        Id,
    },
};
use twilight_validate::request::{audit_reason as validate_audit_reason, ValidationError};

#[derive(Serialize)]
struct UpdateAutoModerationRuleFields<'a> {
    actions: Option<&'a [AutoModerationAction]>,
    enabled: Option<bool>,
    event_type: Option<AutoModerationEventType>,
    exempt_channels: Option<&'a [Id<ChannelMarker>]>,
    exempt_roles: Option<&'a [Id<RoleMarker>]>,
    name: Option<&'a str>,
    trigger_metadata: Option<&'a AutoModerationTriggerMetadata>,
}

/// Update an auto moderation rule in a guild.
///
/// Requires the [`MANAGE_GUILD`] permission.
///
/// [`MANAGE_GUILD`]: twilight_model::guild::Permissions::MANAGE_GUILD
#[must_use = "requests must be configured and executed"]
pub struct UpdateAutoModerationRule<'a> {
    auto_moderation_rule_id: Id<AutoModerationRuleMarker>,
    fields: UpdateAutoModerationRuleFields<'a>,
    guild_id: Id<GuildMarker>,
    http: &'a Client,
    reason: Option<&'a str>,
}

impl<'a> UpdateAutoModerationRule<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        guild_id: Id<GuildMarker>,
        auto_moderation_rule_id: Id<AutoModerationRuleMarker>,
    ) -> Self {
        Self {
            auto_moderation_rule_id,
            fields: UpdateAutoModerationRuleFields {
                actions: None,
                enabled: None,
                event_type: None,
                exempt_channels: None,
                exempt_roles: None,
                name: None,
                trigger_metadata: None,
            },
            guild_id,
            http,
            reason: None,
        }
    }

    /// Set the list of actions.
    pub const fn actions(mut self, actions: &'a [AutoModerationAction]) -> Self {
        self.fields.actions = Some(actions);

        self
    }

    /// Set whether the rule is enabled.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.fields.enabled = Some(enabled);

        self
    }

    /// Set the channels where the rule does not apply.
    pub const fn exempt_channels(mut self, exempt_channels: &'a [Id<ChannelMarker>]) -> Self {
        self.fields.exempt_channels = Some(exempt_channels);

        self
    }

    /// Set the roles to which the rule does not apply.
    pub const fn exempt_roles(mut self, exempt_roles: &'a [Id<RoleMarker>]) -> Self {
        self.fields.exempt_roles = Some(exempt_roles);

        self
    }

    /// Set the trigger metadata.
    ///
    /// Care must be taken to set the correct metadata based on the rule's type.
    pub const fn trigger_metadata(
        mut self,
        trigger_metadata: &'a AutoModerationTriggerMetadata,
    ) -> Self {
        self.fields.trigger_metadata = Some(trigger_metadata);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<AutoModerationRule> {
        let http = self.http;

        match self.try_into_request() {
            Ok(request) => http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl<'a> AuditLogReason<'a> for UpdateAutoModerationRule<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, ValidationError> {
        validate_audit_reason(reason)?;

        self.reason.replace(reason);

        Ok(self)
    }
}

impl TryIntoRequest for UpdateAutoModerationRule<'_> {
    fn try_into_request(self) -> Result<Request, HttpError> {
        let mut request = Request::builder(&Route::UpdateAutoModerationRule {
            auto_moderation_rule_id: self.auto_moderation_rule_id.get(),
            guild_id: self.guild_id.get(),
        })
        .json(&self.fields)?;

        if let Some(reason) = self.reason {
            let header = request::audit_header(reason)?;

            request = request.headers(header);
        }

        Ok(request.build())
    }
}
