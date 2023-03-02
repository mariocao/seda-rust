use serde::{Deserialize, Serialize};

use crate::p2p::P2PMessage;

pub type EventId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventData {
    // Tick types
    BatchChainTick,
    ChainTick,
    P2PMessage(P2PMessage),
    CliCall(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id:   EventId,
    pub data: EventData,
}

impl Event {
    pub fn new<T: ToString>(id: T, data: EventData) -> Self {
        Self {
            id: id.to_string(),
            data,
        }
    }
}
