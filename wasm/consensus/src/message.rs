use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMessage {
    pub batch_header:       Vec<u8>,
    pub bn254_public_key:   Vec<u8>,
    pub signature:          Vec<u8>,
    pub ed25519_public_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Batch(BatchMessage),
}

// TODO: impl Bytes Trait
// impl Message {
//     pub fn to_bytes(&self) -> Vec<u8> {
//         serde_json::to_vec(self).expect("Failed to convert to json bytes")
//     }

//     pub fn from_bytes(bytes: &[u8]) -> Self {
//         serde_json::from_slice(bytes).expect("Failed to get message from json
// bytes")     }
// }

impl FromStr for Message {
    // TODO: Error handling for consensus
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s).expect("Failed to read from json string"))
    }
}
