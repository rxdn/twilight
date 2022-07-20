use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// Gateway close event codes.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq, Serialize_repr)]
#[non_exhaustive]
#[repr(u16)]
pub enum CloseCode {
    /// An unknown error occurred.
    UnknownError = 4000,
    /// An invalid opcode or payload for an opcode was sent.
    UnknownOpcode = 4001,
    /// An invalid payload was sent.
    DecodeError = 4002,
    /// A payload was sent prior to identifying.
    NotAuthenticated = 4003,
    /// An invalid token was sent when identifying.
    AuthenticationFailed = 4004,
    /// Multiple identify payloads were sent.
    AlreadyAuthenticated = 4005,
    /// An invalid sequence was sent for resuming.
    InvalidSequence = 4007,
    /// Too many payloads were sent in a certain amount of time.
    RateLimited = 4008,
    /// The session timed out.
    SessionTimedOut = 4009,
    /// An invalid shard was sent when identifying.
    InvalidShard = 4010,
    /// Sharding is required because there are too many guilds.
    ShardingRequired = 4011,
    /// An invalid version for the gateway was sent.
    InvalidApiVersion = 4012,
    /// An invalid intent was sent.
    InvalidIntents = 4013,
    /// A disallowed intent was sent, may need allowlisting.
    DisallowedIntents = 4014,
}

#[derive(Debug, PartialEq)]
pub struct CloseCodeConversionError {
    code: u16,
}

impl CloseCodeConversionError {
    const fn new(code: u16) -> Self {
        Self { code }
    }

    pub const fn code(&self) -> u16 {
        self.code
    }
}

impl Display for CloseCodeConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.code, f)?;

        f.write_str(" isn't a valid close code")
    }
}

impl Error for CloseCodeConversionError {}

impl TryFrom<u16> for CloseCode {
    type Error = CloseCodeConversionError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let close_code = match value {
            4000 => CloseCode::UnknownError,
            4001 => CloseCode::UnknownOpcode,
            4002 => CloseCode::DecodeError,
            4003 => CloseCode::NotAuthenticated,
            4004 => CloseCode::AuthenticationFailed,
            4005 => CloseCode::AlreadyAuthenticated,
            4007 => CloseCode::InvalidSequence,
            4008 => CloseCode::RateLimited,
            4009 => CloseCode::SessionTimedOut,
            4010 => CloseCode::InvalidShard,
            4011 => CloseCode::ShardingRequired,
            4012 => CloseCode::InvalidApiVersion,
            4013 => CloseCode::InvalidIntents,
            4014 => CloseCode::DisallowedIntents,
            _ => return Err(CloseCodeConversionError::new(value)),
        };

        Ok(close_code)
    }
}

#[cfg(test)]
mod tests {
    use super::CloseCode;
    use serde_test::Token;

    #[test]
    fn variants() {
        serde_test::assert_tokens(&CloseCode::UnknownError, &[Token::U16(4000)]);
        serde_test::assert_tokens(&CloseCode::UnknownOpcode, &[Token::U16(4001)]);
        serde_test::assert_tokens(&CloseCode::DecodeError, &[Token::U16(4002)]);
        serde_test::assert_tokens(&CloseCode::NotAuthenticated, &[Token::U16(4003)]);
        serde_test::assert_tokens(&CloseCode::AuthenticationFailed, &[Token::U16(4004)]);
        serde_test::assert_tokens(&CloseCode::AlreadyAuthenticated, &[Token::U16(4005)]);
        serde_test::assert_tokens(&CloseCode::InvalidSequence, &[Token::U16(4007)]);
        serde_test::assert_tokens(&CloseCode::RateLimited, &[Token::U16(4008)]);
        serde_test::assert_tokens(&CloseCode::SessionTimedOut, &[Token::U16(4009)]);
        serde_test::assert_tokens(&CloseCode::InvalidShard, &[Token::U16(4010)]);
        serde_test::assert_tokens(&CloseCode::ShardingRequired, &[Token::U16(4011)]);
        serde_test::assert_tokens(&CloseCode::InvalidApiVersion, &[Token::U16(4012)]);
        serde_test::assert_tokens(&CloseCode::InvalidIntents, &[Token::U16(4013)]);
        serde_test::assert_tokens(&CloseCode::DisallowedIntents, &[Token::U16(4014)]);
    }

    #[test]
    fn conversion() {
        assert_eq!(CloseCode::try_from(4000).unwrap(), CloseCode::UnknownError);
        assert_eq!(CloseCode::try_from(4001).unwrap(), CloseCode::UnknownOpcode);
        assert_eq!(CloseCode::try_from(4002).unwrap(), CloseCode::DecodeError);
        assert_eq!(
            CloseCode::try_from(4003).unwrap(),
            CloseCode::NotAuthenticated
        );
        assert_eq!(
            CloseCode::try_from(4004).unwrap(),
            CloseCode::AuthenticationFailed
        );
        assert_eq!(
            CloseCode::try_from(4005).unwrap(),
            CloseCode::AlreadyAuthenticated
        );
        assert_eq!(
            CloseCode::try_from(4007).unwrap(),
            CloseCode::InvalidSequence
        );
        assert_eq!(CloseCode::try_from(4008).unwrap(), CloseCode::RateLimited);
        assert_eq!(
            CloseCode::try_from(4009).unwrap(),
            CloseCode::SessionTimedOut
        );
        assert_eq!(CloseCode::try_from(4010).unwrap(), CloseCode::InvalidShard);
        assert_eq!(
            CloseCode::try_from(4011).unwrap(),
            CloseCode::ShardingRequired
        );
        assert_eq!(
            CloseCode::try_from(4012).unwrap(),
            CloseCode::InvalidApiVersion
        );
        assert_eq!(
            CloseCode::try_from(4013).unwrap(),
            CloseCode::InvalidIntents
        );
        assert_eq!(
            CloseCode::try_from(4014).unwrap(),
            CloseCode::DisallowedIntents
        );
        assert!(CloseCode::try_from(5000).is_err());
    }
}
