pub mod application;
pub mod attachment;
pub mod channel;
pub mod guild;
pub mod scheduled_event;
pub mod sticker;
pub mod template;
pub mod user;

mod audit_reason;
mod base;
mod get_gateway;
mod get_gateway_authed;
mod get_user_application;
mod get_voice_regions;
mod multipart;
mod try_into_request;

pub use self::{
    audit_reason::AuditLogReason,
    base::{Request, RequestBuilder},
    get_gateway::GetGateway,
    get_gateway_authed::GetGatewayAuthed,
    get_user_application::GetUserApplicationInfo,
    get_voice_regions::GetVoiceRegions,
    multipart::Form,
    try_into_request::TryIntoRequest,
};
pub use twilight_http_ratelimiting::request::Method;

use crate::error::{Error, ErrorType};
use hyper::header::{HeaderName, HeaderValue};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;
use std::iter;

/// Name of the audit log reason header.
const REASON_HEADER_NAME: &str = "x-audit-log-reason";

/// Type that either serializes to null or a value.
///
/// This is particularly useful when combined with an `Option` by allowing three
/// states via `Option<Nullable<T>>`: undefined, null, and T.
///
/// When the request value is `None` it can skip serialization, while if
/// `Nullable` is provided with `None` within it then it will serialize as
/// null. This mechanism is primarily used in patch requests.
#[derive(Serialize)]
struct Nullable<T>(Option<T>);

fn audit_header(reason: &str) -> Result<impl Iterator<Item = (HeaderName, HeaderValue)>, Error> {
    let header_name = HeaderName::from_static(REASON_HEADER_NAME);
    let encoded_reason = utf8_percent_encode(reason, NON_ALPHANUMERIC).to_string();
    let header_value = HeaderValue::from_str(&encoded_reason).map_err(|e| Error {
        kind: ErrorType::CreatingHeader {
            name: encoded_reason,
        },
        source: Some(Box::new(e)),
    })?;

    Ok(iter::once((header_name, header_value)))
}
