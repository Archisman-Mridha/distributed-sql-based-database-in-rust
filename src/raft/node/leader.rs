use crate::result::Result;
use super::{GenericNode, Role};

/*
  Once a leader has been elected, it begins servicing client requests. Each client request contains
  a command to be executed by the replicated state machines. The leader appends the command to its
  log as a new entry, then issues AppendEntries RPCs in parallel to each of the other servers to
  replicate the entry. When the entry has been safely replicated (to majority of the followers), the
  leader applies the entry to its state machine and returns the result of that execution to the
  client.

  If followers crash or run slowly, or if network packets are lost, the leader retries AppendEntries
  RPCs indefinitely (even after it has responded to the client) until all followers eventually store
  all log entries.
*/
#[derive(Default)]
pub struct Leader { }

impl Leader {
  pub fn new( ) -> Self {
    Self { }
  }
}

impl Role for Leader { }

impl GenericNode<Leader> {
  // Broadcasts a heartbeat to all peers.
  pub fn broadcastHeartbeat(&mut self) -> Result<( )> {
    unimplemented!( )
  }
}