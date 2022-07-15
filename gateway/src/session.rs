//! Active gateway session details.

use serde::{Deserialize, Serialize};
use std::mem;

/// Gateway session information, often used to re-create a shard and resume its
/// session.
///
/// TODO explain all about sessions
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Session {
    /// ID of the gateway session.
    id: String,
    /// Sequence of the most recently received gateway event.
    ///
    /// The first sequence of a session is always 1.
    sequence: u64,
}

impl Session {
    /// Create new configuration for resuming a gateway session.
    ///
    /// Can be provided to [`ConfigBuilder::session`].
    ///
    /// [`ConfigBuilder::session`]: crate::ConfigBuilder::session
    pub const fn new(sequence: u64, session_id: String) -> Self {
        Self {
            sequence,
            id: session_id,
        }
    }

    /// ID of the session being resumed.
    ///
    /// The ID of the session is different from the [ID of the shard]; shards are
    /// identified by an index, and when authenticated with the gateway the shard
    /// is given a unique identifier for the gateway session.
    ///
    /// Session IDs are obtained by shards via sending an [`Identify`] command
    /// with the shard's authentication details, and in return the session ID is
    /// provided via the [`Ready`] event.
    ///
    /// [`Identify`]: twilight_model::gateway::payload::outgoing::Identify
    /// [`Ready`]: twilight_model::gateway::payload::incoming::Ready
    /// [ID of the shard]: crate::ShardId
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Current sequence of the connection.
    ///
    /// Number of the events that have been received during this session. A
    /// larger number typically correlates that the shard has been connected
    /// with this session for a longer time, while a smaller number typically
    /// correlates to meaning that it's been connected with this session for a
    /// shorter duration of time.
    ///
    /// As a shard is connected to the gateway and receives events this sequence
    /// will be updated in real time when obtaining the [session of a shard].
    ///
    /// [session of a shard]: crate::Shard::session
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Set the sequence, returning the previous sequence.
    pub(crate) fn set_sequence(&mut self, sequence: u64) -> u64 {
        mem::replace(&mut self.sequence, sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::assert_impl_all;
    use std::fmt::Debug;

    assert_impl_all!(
        Session: Clone,
        Debug,
        Deserialize<'static>,
        Eq,
        PartialEq,
        Send,
        Serialize,
        Sync
    );

    /// Test that sessions deserialize and serialize the same way.
    #[test]
    fn serde() {
        const SEQUENCE: u64 = 56_132;
        const SESSION_ID: &str = "thisisanid";

        let value = Session::new(SEQUENCE, SESSION_ID.to_owned());

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Session",
                    len: 2,
                },
                Token::Str("id"),
                Token::Str(SESSION_ID),
                Token::Str("sequence"),
                Token::U64(SEQUENCE),
                Token::StructEnd,
            ],
        );
    }

    /// Test that session getters return the provided values.
    #[test]
    fn session() {
        const SESSIONS: [(u64, &str); 2] = [(1, "a"), (2, "b")];

        for (sequence, session_id) in SESSIONS {
            let session = Session::new(sequence, session_id.to_owned());
            assert_eq!(session.sequence(), sequence);
            assert_eq!(session.id(), session_id);
        }
    }

    /// Test that setting the sequence actually updates the sequence and returns
    /// the previous sequence.
    #[test]
    fn set_sequence() {
        const SEQUENCE_INITIAL: u64 = 1;
        const SEQUENCE_NEXT: u64 = SEQUENCE_INITIAL + 1;
        const SEQUENCE_SKIPPED: u64 = SEQUENCE_NEXT + 3;

        let mut session = Session::new(SEQUENCE_INITIAL, String::new());
        let old = session.set_sequence(SEQUENCE_NEXT);
        assert_eq!(old, SEQUENCE_INITIAL);

        // although we don't expect to skip sequences the setter should still
        // handle them as usual
        let skipped_old = session.set_sequence(SEQUENCE_SKIPPED);
        assert_eq!(skipped_old, SEQUENCE_NEXT);
    }
}
