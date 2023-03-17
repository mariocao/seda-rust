use near_sdk::{env, near_bindgen};

use crate::{manage_storage_deposit, merkle::merklize, MainchainContract, MainchainContractExt};

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    #[payable]
    pub fn post_data_request(&mut self, data_request: String) {
        manage_storage_deposit!(self, "require", self.data_request_accumulator.push(&data_request));
    }

    pub fn compute_merkle_root(&self) -> Vec<u8> {
        merklize(&self.data_request_accumulator.to_vec()).0.try_into().unwrap()
    }
}
