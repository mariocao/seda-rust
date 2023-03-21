use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageKind {
    Batch,
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageKind::Batch => write!(f, "batch"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message: Vec<u8>,
    pub kind:    MessageKind,
}

// TODO: impl Bytes Trait
impl Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to convert to json bytes")
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_slice(bytes).expect("Failed to get message from json bytes")
    }
}

impl FromStr for Message {
    // TODO: Error handling for consensus
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s).expect("Failed to read from json string"))
    }
}
