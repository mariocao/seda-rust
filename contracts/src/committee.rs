use near_sdk::{near_bindgen, AccountId};

use crate::{consts::SLOTS_PER_EPOCH, MainchainContract, MainchainContractExt};

/// Contract private methods
impl MainchainContract {
    pub fn select_committee(&mut self) -> Vec<AccountId> {
        // TODO: committee selection algorithm
        // temp: fill the committee with active nodes round robin style
        let mut committee = vec![];
        let active_nodes_vec = self.active_nodes.keys_as_vector();
        for i in 0..SLOTS_PER_EPOCH {
            committee.push(active_nodes_vec.get(i % active_nodes_vec.len()).unwrap());
        }
        committee
    }
}

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    pub fn get_committees(&self) -> Vec<Vec<AccountId>> {
        self.committees.clone()
    }
}
