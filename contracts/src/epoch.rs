use near_sdk::{env, log, near_bindgen};

use crate::{
    consts::{EPOCH_COMMITTEES_LOOKAHEAD, NEAR_BLOCKS_PER_SEDA_SLOT, SLOTS_PER_EPOCH},
    manage_storage_deposit,
    MainchainContract,
    MainchainContractExt,
};

pub type EpochHeight = u64;

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    pub fn get_current_epoch(&self) -> u64 {
        env::block_height() / (NEAR_BLOCKS_PER_SEDA_SLOT * SLOTS_PER_EPOCH)
    }

    #[payable]
    pub fn process_epoch(&mut self) {
        manage_storage_deposit!(self, {
            // check if epoch has already been processed
            let epoch = self.get_current_epoch();
            if epoch <= self.last_processed_epoch {
                log!("Epoch {} has already been processed", epoch);
                return;
            }
            log!("Processing epoch {}", epoch);

            // move pending nodes to active nodes if they are eligible for this epoch
            let eligible_nodes: Vec<_> = self
                .pending_nodes
                .iter()
                .filter(|(_, activation_epoch)| activation_epoch <= &epoch)
                .map(|(account_id, _)| account_id)
                .collect();
            for account_id in eligible_nodes {
                log!("Moving pending node {} to active nodes", account_id);
                self.active_nodes
                    .insert(&account_id, &self.inactive_nodes.get(&account_id).unwrap());
                self.inactive_nodes.remove(&account_id);
                self.pending_nodes.remove(&account_id);
            }
            // log!("pending_nodes: {:?}", self.pending_nodes.to_vec());

            // if bootstrapping phase, wait until there are committee_size active nodes
            if self.bootstrapping_phase {
                if self.active_nodes.len() < self.config.committee_size {
                    log!("Not enough active nodes to exit bootstrapping phase");
                    return;
                }
                self.bootstrapping_phase = false;
                log!("Exiting bootstrapping phase");
                // select committees EPOCH_COMMITTEES_LOOKAHEAD epochs in advance
                for i in 0..EPOCH_COMMITTEES_LOOKAHEAD {
                    let committee = self.select_committee(self.last_generated_random_number);
                    self.committees.insert(&(epoch + i), &committee);
                }
            }

            // select committee from active nodes
            let committee = self.select_committee(self.last_generated_random_number);
            log!("Selected committee for epoch {}: {:?}", epoch, committee);
            self.committees
                .insert(&(epoch + EPOCH_COMMITTEES_LOOKAHEAD), &committee);

            // set last processed epoch to current epoch
            self.last_processed_epoch = epoch;
        });
    }
}
