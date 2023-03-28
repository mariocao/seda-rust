use near_sdk::{env, near_bindgen};

use crate::{
    consts::{NEAR_BLOCKS_PER_SEDA_SLOT, SLOTS_PER_EPOCH},
    MainchainContract,
    MainchainContractExt,
};

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    pub fn get_current_slot(&self) -> u64 {
        env::block_height() / NEAR_BLOCKS_PER_SEDA_SLOT
    }

    pub fn get_current_slot_within_epoch(&self) -> u64 {
        self.get_current_slot() % SLOTS_PER_EPOCH
    }
}
