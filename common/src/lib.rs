mod node;
use borsh::{BorshDeserialize, BorshSerialize};
pub use node::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug, Clone)]
pub struct HumanReadableDepositInfo {
    pub node_ed25519_public_key: Vec<u8>,
    pub amount:                  u128,
}
