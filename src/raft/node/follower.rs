use std::collections::HashSet;
use tokio::sync::mpsc::UnboundedSender;
use crate::{
  raft::{
    log::Log, message::Message, state_machine_driver::StateMachineInstruction,
    types::{NodeId, Ticks}
  },
  result::Result
};
use super::{getRandomElectionTimeout, GenericNode, Role};

/*
  A follower replicates state from the leader.

  The node becomes a follower when it -
  (a) Loses the election
  (b) Discovers a new term and enters into it as a leaderless follower (since it doesn't know who
      the leader is).

  When servers start up, they begin as followers. A server remains in follower state as long as it
  receives valid RPCs from a leader or candidate. Leaders send periodic heartbeats to all followers
  in order to maintain their authority. If a follower receives no communication over a period of
  time (called the election timeout), then it assumes there is no leader and begins an election to
  choose a new leader.
*/
#[derive(Default)]
pub struct Follower {
  leader: Option<NodeId>,

  // Cast vote represents the node that this node voted for in the current term.
  castVote: Option<NodeId>,

  timeSinceLeaderSentHeartbeat: Ticks,
  electionTimeout: Ticks,

  // Id of requests sent by the client, directly to this node.
  // NOTE : These requests are forwarded to the leader / rejected during leader or term change.
  pub(in crate::raft) requestsFromClient: HashSet<Vec<u8>>
}

impl Follower {
  pub fn new(leader: Option<u8>, castVote: Option<u8>) -> Self {
    Self {
      leader,
      castVote,

      electionTimeout: getRandomElectionTimeout( ),

      ..Default::default( )
    }
  }
}

impl Role for Follower { }

impl GenericNode<Follower> {
  pub fn newAsLeaderless(nodeId: u8,
                         peers: HashSet<u8>,
                         mut log: Log,
                         messageSender: UnboundedSender<Message>,
                         stateMachineDriverInstructionsSender: UnboundedSender<StateMachineInstruction>) -> Result<GenericNode>
  {
    let (newlyDiscoveredTerm, castVoteInNewlyDiscoveredTerm)= log.getCurrentTermAndCastVote( )?;

    Ok(GenericNode {
      role: Follower::new(None, castVoteInNewlyDiscoveredTerm),
      currentTerm: newlyDiscoveredTerm,

      id: nodeId,
      peers,
      messageSender,

      log,
      stateMachineInstructor: stateMachineDriverInstructionsSender
    })
  }
}
