use rand::{thread_rng, Rng};
use std::collections::HashSet;
use tokio::sync::mpsc::UnboundedSender;
use candidate::Candidate;
use follower::Follower;
use leader::Leader;
use super::{
  log::Log, message::Message, state_machine_driver::StateMachineDriverInstruction,
  types::{NodeId, Term, Ticks}
};
use std::ops::Range;

pub enum Node {
  Candidate(GenericNode<Candidate>),
  Follower(GenericNode<Follower>),
  Leader(GenericNode<Leader>)
}

pub mod follower;
pub mod candidate;
pub mod leader;

pub struct GenericNode<R: Role= Follower> {
  role: R,
  currentTerm: Term,

  id: NodeId,
  peers: HashSet<NodeId>,
  messageSender: UnboundedSender<Message>,

  log: Log,

  // Sends instruction to the state-machine driver.
  stateMachineDriverInstructionsSender: UnboundedSender<StateMachineDriverInstruction>
}

impl<R: Role> GenericNode<R> {
  // Changes the node's role.
  fn changeRole<NR: Role>(self, newRole: NR) -> GenericNode<NR> {
    GenericNode {
      role: newRole,
      currentTerm: self.currentTerm,

      id: self.id,
      peers: self.peers,
      messageSender: self.messageSender,

      log: self.log,
      stateMachineDriverInstructionsSender: self.stateMachineDriverInstructionsSender
    }
  }

  // Returns the cluster-size (number of nodes in the cluster).
  fn clusterSize(&self) -> u8 {
    let peerCount= self.peers.len( ) as u8;
    peerCount + 1
  }

  // A quorum refers to the minimum number of group members in a decision-making process that must
  // be present in order for the proceedings to be valid.
  fn quorom(&self) -> u8 {
    getQuorumForClusterSize(self.clusterSize( ))
  }
}

pub trait Role { }

fn getQuorumForClusterSize(clusterSize: u8) -> u8 {
  (clusterSize / 2) + 1
}

/*
  Raft uses randomized election timeouts to ensure that split votes are rare and that they are
  resolved quickly. To prevent split votes in the first place, election timeouts are chosen randomly
  from a fixed interval (e.g. 150 - 300 ms).

  In most cases, only a single server will timeout.

  Also each candidate restarts its randomized election timeout at the start of an election, and it
  waits for that timeout to elapse before starting the next election; this reduces the likelihood of
  another split vote in the new election.
*/
const ELECTION_TIMEOUT_RANGE: Range<Ticks> = 10..20;

// Generates a random election timeout within range (10 - 20 ms).
fn getRandomElectionTimeout( ) -> Ticks {
  thread_rng( )
    .gen_range(ELECTION_TIMEOUT_RANGE)
}