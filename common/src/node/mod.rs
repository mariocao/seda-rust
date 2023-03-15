mod get;
pub use get::*;

mod register;
pub use register::*;

mod update;
pub use update::*;

use super::*;

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct HumanReadableNode {
    pub account_id:       near_sdk::AccountId,
    pub multi_addr:       String,
    pub balance:          u128,
    pub bn254_public_key: Vec<u8>,
}
