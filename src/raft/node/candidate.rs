use super::{follower::Follower, getRandomElectionTimeout, GenericNode, Role};
use crate::{raft::{node::leader::Leader, types::{NodeId, Term, Ticks}}, result::Result};
use std::collections::HashSet;
use tracing::info;

/*
  To begin an election, a follower increments its current term and transitions to candidate state.
  It then votes for itself and issues RequestVote RPCs in parallel to each of the other servers in
  the cluster. A candidate continues in this state until one of three things happens:
  
  (a) it wins the election
      NOTE : Each server will vote for at most one candidate in a given term, on a
              first-come-first-served basis.
  (b) another server establishes itself as leader
  (c) a period of time goes by with no winner.
*/
#[derive(Default)]
pub struct Candidate {
  // Time elapsed since the election started.
  electionDuration: Ticks,

  // Election timeout = Time when the election started - Time when the election will end.
  electionTimeout: Ticks,

  receivedVotes: HashSet<NodeId>,
}

impl Candidate {
  pub fn new( ) -> Self {
    Self {
      electionTimeout: getRandomElectionTimeout( ),
      ..Default::default( )
    }
  }
}

impl Role for Candidate { }

impl GenericNode<Candidate> {
  // Start new term and campaign for leadership.
  pub(in crate::raft) fn startNewTerm(&mut self) -> Result<( )> {
    let newTerm = self.currentTerm + 1;
    info!("Starting campaign for new term {}", newTerm);

    self.currentTerm = newTerm;
    self.role = Candidate::new( );
    self.role.receivedVotes.insert(self.id); // Node votes for itself.

    let castVote= Some(self.id);
    self.log.setCurrentTermAndCastVote(newTerm, castVote);

    todo!( )
  }

  //
  pub(in crate::raft) fn becomeLeader(mut self) -> Result<GenericNode<Leader>> {
    info!("Won election in term {} | Becoming leader", self.currentTerm);

    unimplemented!( );

    let node= self.changeRole(Leader::new( ));
    unimplemented!( );

    Ok(node)
  }

  /*
    Transitions the node from a candidate to a follower.

    The node becomes a follower when it -
    (a) Loses the election
    (b) Discovers a new term and enters into it as a leaderless follower (since it doesn't know who
        the leader is).
  */
  pub(in crate::raft) fn becomeFollower(mut self,
                                        currentTerm: Term,
                                        leader: Option<NodeId>) -> Result<GenericNode<Follower>>
  {
    assert!(currentTerm >= self.currentTerm,
            "Term transition attemp from {} to {}", self.currentTerm, currentTerm);

    match leader {
      // CASE (a) - The node lost the election.
      Some(leader) => {
        assert_eq!(currentTerm, self.currentTerm, "Can't follow leader in a different term");

        info!("Lost election in the current term {} | Following leader {}", currentTerm, leader);

        let castVote = Some(self.id);
        Ok(self.changeRole(Follower::new(Some(leader), castVote)))
      }

      // CASE (b) - The node discovered a new term (in which case it'll step into the term as a
      // leaderless follower).
      None => {
        let previousTerm = self.currentTerm;
        assert_ne!(currentTerm, previousTerm, "Can't become leaderless follower in the current term");

        info!("Discovered new term {} | Becoming a leaderless follower", currentTerm);

        self.currentTerm = currentTerm;
        self.log.setCurrentTermAndCastVote(currentTerm, None);

        Ok(self.changeRole(Follower::new(None, None)))
      }
    }
  }
}