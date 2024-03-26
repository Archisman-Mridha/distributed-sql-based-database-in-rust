use super::types::Term;

// Represents a message exchanged between nodes.
pub struct Message {
  pub currentTermOfSender: Term,

  pub from: MessageAddress,
  pub to: MessageAddress,

  pub payload: MessagePayload
}

pub enum MessageAddress { }

pub enum MessagePayload {
  // Represents the periodic heartbeat sent from leader to its followers.
  Heartbeat { },

  ClientRequest { },

  ResponseToClient { }
}