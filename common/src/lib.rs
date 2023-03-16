mod node;
use borsh::{BorshDeserialize, BorshSerialize};
pub use node::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct DepositInfo {
    pub node_ed25519_public_key: Vec<u8>,
    pub amount:                  u128,
}
