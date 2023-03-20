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

/// Withdraw request info for one account to a node
#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct WithdrawRequest {
    pub amount: u128,
    pub epoch:  u64, // epoch when funds will be available for withdrawal
}
