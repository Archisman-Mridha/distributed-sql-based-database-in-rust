use crate::{result::Result, storage::engine::StorageEngine};
use super::types::{LogEntryIndex, NodeId, Term};

/*
  Represents the distributed immutable append-only commit log.

  Each log entry stores a state machine command along with the term number when the entry was
  received by the leader. The term numbers in log entries are used to detect inconsistencies between
  logs. Each log entry also has an integer index identifying its position in the log.

  The leader decides when it is safe to apply a log entry to the state machines - such an entry is
  called committed. This also commits all preceding entries in the leader’s log, including entries
  created by previous leaders.

  The leader keeps track of the highest index it knows to be committed, and it includes that index
  in future AppendEntries RPCs (including heartbeats) so that the other servers eventually find out.
  Once a follower learns that a log entry is committed, it applies the entry to its local state
  machine (in log order).
*/
pub struct Log {
  storageEngine: Box<dyn StorageEngine>,

  // Index of the last stored entry.
  lastStoredEntryIndex: LogEntryIndex,

  // Active term when the last entry was stored.
  lastStoredEntryTerm: Term
}

impl Log {
  pub fn setCurrentTermAndCastVote(&mut self, term: Term, castVote: Option<NodeId>) -> Result<( )> {
    unimplemented!( )
  }

  pub fn getCurrentTermAndCastVote(&mut self) -> Result<(Term, Option<NodeId>)> {
    unimplemented!( )
  }

  pub fn getLastStoredEntryIndexAndTerm(&self) -> (LogEntryIndex, Term) {
    (self.lastStoredEntryIndex, self.lastStoredEntryTerm)
  }
}