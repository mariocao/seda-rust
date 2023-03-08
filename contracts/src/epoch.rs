use near_sdk::{env, log, near_bindgen, AccountId};

use crate::{
    consts::{EPOCH_COMMITTEES_LOOKAHEAD, INIT_COMMITTEE_SIZE, NEAR_BLOCKS_PER_SEDA_SLOT, SLOTS_PER_EPOCH},
    MainchainContract,
    MainchainContractExt,
};

pub type EpochHeight = u64;

/// Contract private methods
impl MainchainContract {
    fn select_committee(&mut self) -> Vec<AccountId> {
        // temporarily select the first config.committee_size nodes from active nodes
        // TODO: implement committee selection algorithm
        let mut committee = vec![];
        for (account_id, _) in self.active_nodes.iter() {
            if committee.len() as u64 == self.config.committee_size {
                break;
            }
            committee.push(account_id.clone());
        }
        committee
    }
}

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    pub fn get_current_epoch(&self) -> u64 {
        env::block_height() / (NEAR_BLOCKS_PER_SEDA_SLOT * SLOTS_PER_EPOCH)
    }

    pub fn process_epoch(&mut self) {
        // check if epoch has already been processed
        if self.get_current_epoch() <= self.last_processed_epoch {
            log!("Epoch has already been processed");
            return;
        }

        // if bootstrapping phase, wait until there are INIT_COMMITTEE_SIZE active nodes
        if self.bootstrapping_phase {
            if self.active_nodes.len() < INIT_COMMITTEE_SIZE {
                log!("Not enough active nodes to exit bootstrapping phase");
                return;
            }
            self.bootstrapping_phase = false;
            // select committees EPOCH_COMMITTEES_LOOKAHEAD epochs in advance
            for _ in 1..EPOCH_COMMITTEES_LOOKAHEAD {
                let committee = self.select_committee();
                self.committees.push(committee);
            }
        }

        // move pending nodes to active nodes if they are eligible for this epoch
        self.pending_nodes.to_vec().retain(|(account_id, activation_epoch)| {
            if activation_epoch == &self.get_current_epoch() {
                self.active_nodes
                    .insert(account_id, &self.inactive_nodes.get(account_id).unwrap());
                false
            } else {
                true
            }
        });

        // select committee from active nodes
        let committee = self.select_committee();
        self.committees.push(committee);

        // set last processed epoch to current epoch
        self.last_processed_epoch = self.get_current_epoch();
    }
}
