use near_sdk::{env, near_bindgen};

use crate::{MainchainContract, MainchainContractExt};
use crate::consts::NEAR_BLOCKS_PER_SEDA_SLOT;

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    pub fn get_current_slot(&self) -> u64 {
        env::block_height() / NEAR_BLOCKS_PER_SEDA_SLOT
    }
}
