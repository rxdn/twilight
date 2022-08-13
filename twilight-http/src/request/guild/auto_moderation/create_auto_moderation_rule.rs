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
        AutoModerationActionType, AutoModerationEventType, AutoModerationKeywordPresetType,
        AutoModerationRule, AutoModerationTriggerType,
    },
    id::{
        marker::{ChannelMarker, GuildMarker, RoleMarker},
        Id,
    },
};
use twilight_validate::request::{audit_reason as validate_audit_reason, ValidationError};

#[derive(Serialize)]
struct CreateAutoModerationRuleFieldsAction {
    /// Type of action.
    pub kind: AutoModerationActionType,
    /// Additional metadata needed during execution for this specific action
    /// type.
    pub metadata: CreateAutoModerationRuleFieldsActionMetadata,
}

#[derive(Default, Serialize)]
struct CreateAutoModerationRuleFieldsActionMetadata {
    /// Channel to which user content should be logged.
    pub channel_id: Option<Id<ChannelMarker>>,
    /// Timeout duration in seconds.
    ///
    /// Maximum value is 2419200 seconds, or 4 weeks.
    pub duration_seconds: Option<u32>,
}

#[derive(Serialize)]
struct CreateAutoModerationRuleFieldsTriggerMetadata<'a> {
    allow_list: Option<&'a [&'a str]>,
    keyword_filter: Option<&'a [&'a str]>,
    presets: Option<&'a [AutoModerationKeywordPresetType]>,
}

#[derive(Serialize)]
struct CreateAutoModerationRuleFields<'a> {
    actions: Option<Vec<CreateAutoModerationRuleFieldsAction>>,
    enabled: Option<bool>,
    event_type: AutoModerationEventType,
    exempt_channels: Option<&'a [Id<ChannelMarker>]>,
    exempt_roles: Option<&'a [Id<RoleMarker>]>,
    name: &'a str,
    trigger_metadata: Option<CreateAutoModerationRuleFieldsTriggerMetadata<'a>>,
    trigger_type: Option<AutoModerationTriggerType>,
}

/// Create an auto moderation rule within a guild.
///
/// Requires the [`MANAGE_GUILD`] permission.
///
/// # Examples
///
/// Create a rule that deletes messages that contain the word "darn":
///
/// ```no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use twilight_http::Client;
/// use twilight_model::{guild::auto_moderation::AutoModerationEventType, id::Id};
///
/// let client = Client::new("my token".to_owned());
///
/// let guild_id = Id::new(1);
/// client
///     .create_auto_moderation_rule(guild_id, "no darns", AutoModerationEventType::MessageSend)
///     .action_block_message()
///     .enabled(true)
///     .with_trigger_keyword(&["darn"])
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`MANAGE_GUILD`]: twilight_model::guild::Permissions::MANAGE_GUILD
#[must_use = "requests must be configured and executed"]
pub struct CreateAutoModerationRule<'a> {
    fields: CreateAutoModerationRuleFields<'a>,
    guild_id: Id<GuildMarker>,
    http: &'a Client,
    reason: Option<&'a str>,
}

impl<'a> CreateAutoModerationRule<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        guild_id: Id<GuildMarker>,
        name: &'a str,
        event_type: AutoModerationEventType,
    ) -> Self {
        Self {
            fields: CreateAutoModerationRuleFields {
                actions: None,
                enabled: None,
                event_type,
                exempt_channels: None,
                exempt_roles: None,
                name,
                trigger_metadata: None,
                trigger_type: None,
            },
            guild_id,
            http,
            reason: None,
        }
    }

    /// Append an action of type [`BlockMessage`].
    ///
    /// [`BlockMessage`]: AutoModerationActionType::BlockMessage
    pub fn action_block_message(mut self) -> Self {
        self.fields.actions.get_or_insert_with(Vec::new).push(
            CreateAutoModerationRuleFieldsAction {
                kind: AutoModerationActionType::BlockMessage,
                metadata: CreateAutoModerationRuleFieldsActionMetadata::default(),
            },
        );

        self
    }

    /// Append an action of type [`SendAlertMessage`].
    ///
    /// [`SendAlertMessage`]: AutoModerationActionType::SendAlertMessage
    pub fn action_send_alert_message(mut self, channel_id: Id<ChannelMarker>) -> Self {
        self.fields.actions.get_or_insert_with(Vec::new).push(
            CreateAutoModerationRuleFieldsAction {
                kind: AutoModerationActionType::SendAlertMessage,
                metadata: CreateAutoModerationRuleFieldsActionMetadata {
                    channel_id: Some(channel_id),
                    ..Default::default()
                },
            },
        );

        self
    }

    /// Append an action of type [`Timeout`].
    ///
    /// [`Timeout`]: AutoModerationActionType::Timeout
    pub fn action_timeout(mut self, duration_seconds: u32) -> Self {
        self.fields.actions.get_or_insert_with(Vec::new).push(
            CreateAutoModerationRuleFieldsAction {
                kind: AutoModerationActionType::Timeout,
                metadata: CreateAutoModerationRuleFieldsActionMetadata {
                    duration_seconds: Some(duration_seconds),
                    ..Default::default()
                },
            },
        );

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

    /// Create the request with the trigger type [`Keyword`], then execute it.
    ///
    /// Rules of this type require the `keyword_filter` field specified, and
    /// this method ensures this. See [Discord Docs/Keyword Matching Strategies]
    /// and [Discord Docs/Trigger Metadata].
    ///
    /// [`Keyword`]: AutoModerationTriggerType::Keyword
    /// [Discord Docs/Keyword Matching Strategies]: https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-keyword-matching-strategies
    /// [Discord Docs/Trigger Metadata]: https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-metadata
    pub fn with_keyword(
        mut self,
        keyword_filter: &'a [&'a str],
    ) -> ResponseFuture<AutoModerationRule> {
        self.fields.trigger_metadata = Some(CreateAutoModerationRuleFieldsTriggerMetadata {
            allow_list: None,
            keyword_filter: Some(keyword_filter),
            presets: None,
        });

        self.fields.trigger_type = Some(AutoModerationTriggerType::Keyword);

        self.exec()
    }

    /// Create the request with the trigger type [`HarmfulLink`], then execute
    /// it.
    ///
    /// [`HarmfulLink`]: AutoModerationTriggerType::HarmfulLink
    pub fn with_harmful_link(mut self) -> ResponseFuture<AutoModerationRule> {
        self.fields.trigger_type = Some(AutoModerationTriggerType::HarmfulLink);

        self.exec()
    }

    /// Create the request with the trigger type [`Spam`], then execute it.
    ///
    /// [`Spam`]: AutoModerationTriggerType::Spam
    pub fn with_spam(mut self) -> ResponseFuture<AutoModerationRule> {
        self.fields.trigger_type = Some(AutoModerationTriggerType::Spam);

        self.exec()
    }

    /// Create the request with the trigger type [`KeywordPreset`], then execute
    /// it.
    ///
    /// Rules of this type require the `presets` field specified, and
    /// this method ensures this. See [Discord Docs/Trigger Metadata].
    ///
    /// [`KeywordPreset`]: AutoModerationTriggerType::KeywordPreset
    /// [Discord Docs/Trigger Metadata]: https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-metadata
    pub fn with_keyword_preset(
        mut self,
        presets: &'a [AutoModerationKeywordPresetType],
        allow_list: &'a [&'a str],
    ) -> ResponseFuture<AutoModerationRule> {
        self.fields.trigger_metadata = Some(CreateAutoModerationRuleFieldsTriggerMetadata {
            allow_list: Some(allow_list),
            keyword_filter: None,
            presets: Some(presets),
        });

        self.fields.trigger_type = Some(AutoModerationTriggerType::KeywordPreset);

        self.exec()
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    fn exec(self) -> ResponseFuture<AutoModerationRule> {
        let http = self.http;

        match self.try_into_request() {
            Ok(request) => http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl<'a> AuditLogReason<'a> for CreateAutoModerationRule<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, ValidationError> {
        validate_audit_reason(reason)?;

        self.reason.replace(reason);

        Ok(self)
    }
}

impl TryIntoRequest for CreateAutoModerationRule<'_> {
    fn try_into_request(self) -> Result<Request, HttpError> {
        let mut request = Request::builder(&Route::CreateAutoModerationRule {
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