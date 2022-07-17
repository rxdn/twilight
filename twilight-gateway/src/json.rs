//! Function wrappers for deserializing and serializing events and commands.

#[cfg(not(feature = "simd-json"))]
pub use serde_json::to_vec;
#[cfg(feature = "simd-json")]
pub use simd_json::to_vec;

#[cfg(not(feature = "simd-json"))]
use serde_json::from_slice as from_slice_inner;
#[cfg(feature = "simd-json")]
use simd_json::from_slice as from_slice_inner;

use serde::de::{DeserializeOwned, DeserializeSeed};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    str::{self, Utf8Error},
};
use twilight_model::gateway::event::GatewayEvent;

#[cfg(not(feature = "simd-json"))]
use twilight_model::gateway::event::GatewayEventDeserializer as EventDeserializer;
#[cfg(feature = "simd-json")]
use twilight_model::gateway::event::GatewayEventDeserializerOwned as EventDeserializer;

/// Parsing of a gateway event failed, likely due to a type being unrecognized.
#[derive(Debug)]
pub struct GatewayEventParsingError {
    /// Type of error.
    pub(crate) kind: GatewayEventParsingErrorType,
    /// Source error if available.
    pub(crate) source: Option<Box<dyn Error + Send + Sync>>,
}

impl GatewayEventParsingError {
    /// Shortcut to create a new error from an invalid UTF-8 error.
    fn from_utf8(source: Utf8Error) -> Self {
        Self {
            kind: GatewayEventParsingErrorType::PayloadInvalid,
            source: Some(Box::new(source)),
        }
    }
}

impl Display for GatewayEventParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.kind {
            GatewayEventParsingErrorType::Deserializing => {
                f.write_str("deserializing gateway event as json failed")
            }
            GatewayEventParsingErrorType::PayloadInvalid => {
                f.write_str("payload is an invalid json structure")
            }
        }
    }
}

impl Error for GatewayEventParsingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`GatewayEventParsingError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GatewayEventParsingErrorType {
    /// Deserializing the GatewayEvent payload from JSON failed.
    Deserializing,
    /// The payload received from Discord was an unrecognized or invalid
    /// structure.
    ///
    /// The payload was either invalid JSON, not UTF-8 valid, or did not contain
    /// the necessary "op" key in the object.
    PayloadInvalid,
}

/// Wrapper over [`from_slice_inner`], wrapping the error type with a custom
/// one.
///
/// # Errors
///
/// Returns a [`GatewayEventParsingErrorType::Deserializing`] error type if the
/// payload failed to deserialize.
pub fn from_slice<T: DeserializeOwned>(json: &mut [u8]) -> Result<T, GatewayEventParsingError> {
    from_slice_inner(json).map_err(|source| {
        tracing::error!("invalid JSON: {}", String::from_utf8_lossy(json));

        GatewayEventParsingError {
            kind: GatewayEventParsingErrorType::Deserializing,
            source: Some(Box::new(source)),
        }
    })
}

/// Parse JSON into a gateway event without existing knowledge of its underlying
/// parts.
///
/// # Errors
///
/// Returns a [`GatewayEventParsingErrorType::Deserializing`] error type if the
/// payload failed to deserialize.
///
/// Returns a [`GatewayEventParsingErrorType::PayloadInvalid`] error type if the
/// payload wasn't a valid `GatewayEvent` data structure, such as due to not
/// being UTF-8 valid.
pub fn parse(json: &mut [u8]) -> Result<GatewayEvent, GatewayEventParsingError> {
    #[cfg(feature = "simd-json")]
    let (gateway_deserializer, mut json_deserializer) = {
        let gateway_deserializer = EventDeserializer::from_json(
            str::from_utf8(json).map_err(GatewayEventParsingError::from_utf8)?,
        )
        .ok_or(GatewayEventParsingError {
            kind: GatewayEventParsingErrorType::PayloadInvalid,
            source: None,
        })?;

        let mut json_deserializer =
            simd_json::Deserializer::from_slice(json).map_err(|_| GatewayEventParsingError {
                kind: GatewayEventParsingErrorType::PayloadInvalid,
                source: None,
            })?;

        (gateway_deserializer, json_deserializer)
    };

    #[cfg(not(feature = "simd-json"))]
    let (gateway_deserializer, mut json_deserializer) = {
        let text = str::from_utf8(json).map_err(GatewayEventParsingError::from_utf8)?;

        let gateway_deserializer =
            EventDeserializer::from_json(text).ok_or(GatewayEventParsingError {
                kind: GatewayEventParsingErrorType::PayloadInvalid,
                source: None,
            })?;

        let json_deserializer = serde_json::Deserializer::from_slice(json);

        (gateway_deserializer, json_deserializer)
    };

    gateway_deserializer
        .deserialize(&mut json_deserializer)
        .map_err(|source| {
            tracing::error!("invalid JSON: {}", String::from_utf8_lossy(json));

            GatewayEventParsingError {
                kind: GatewayEventParsingErrorType::Deserializing,
                source: Some(Box::new(source)),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{GatewayEventParsingError, GatewayEventParsingErrorType};
    use static_assertions::assert_impl_all;
    use std::{error::Error, fmt::Debug};

    assert_impl_all!(GatewayEventParsingErrorType: Debug, Send, Sync);
    assert_impl_all!(GatewayEventParsingError: Error, Send, Sync);
}