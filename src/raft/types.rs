pub type NodeId= u8;

/*
  Raft divides time into terms of arbitrary length. Terms are numbered with consecutive integers.
  Each term begins with an election, in which one or more candidates attempt to become the leader
  by voting to self.

  If a candidate wins the election, then it serves as leader for the rest of the term.

  In some situations an election will result in a split vote. In this case the term will end with no
  leader. A new term (with a new election) will begin shortly.

  Terms act as a logical clock in Raft, and they allow servers to detect obsolete information such
  as stale leaders.

  Current terms are exchanged whenever servers communicate. If one server’s current term is smaller
  than the other’s, then it updates its current term to the larger value. If a candidate or leader
  discovers that its term is out of date, it immediately reverts to follower state. If a server
  receives a request with a stale term number, it rejects the request.
*/
pub type Term= u64;

// Represents a logical clock interval.
pub type Ticks= u8;

pub type LogEntryIndex= u64;