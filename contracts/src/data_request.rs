use near_sdk::{env, near_bindgen};
use seda_common::ComputeMerkleRootResult;

use crate::{manage_storage_deposit, merkle::merklize, MainchainContract, MainchainContractExt};

/// Contract private methods
impl MainchainContract {
    pub fn internal_compute_merkle_root(&self) -> Vec<u8> {
        merklize(&self.data_request_accumulator.to_vec()).0.try_into().unwrap()
    }
}

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    #[payable]
    pub fn post_data_request(&mut self, data_request: String) {
        manage_storage_deposit!(self, "require", self.data_request_accumulator.push(&data_request));
    }

    /// Returns the merkle root of the data request accumulator, the current
    /// slot and the current slot leader`
    pub fn compute_merkle_root(&self) -> ComputeMerkleRootResult {
        ComputeMerkleRootResult {
            merkle_root:         self.internal_compute_merkle_root(),
            current_slot:        self.get_current_slot(),
            current_slot_leader: self.get_current_slot_leader().map(|x| x.to_string()),
        }
    }
}
