use near_sdk::{env, log, near_bindgen};
use rand::Rng;

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
        manage_storage_deposit!(self, "require", {
            // check if epoch has already been processed
            if self.get_current_epoch() <= self.last_processed_epoch {
                log!("Epoch has already been processed");
                return;
            }

            // move pending nodes to active nodes if they are eligible for this epoch
            self.pending_nodes.to_vec().retain(|(account_id, activation_epoch)| {
                if activation_epoch <= &self.get_current_epoch() {
                    self.active_nodes
                        .insert(account_id, &self.inactive_nodes.get(account_id).unwrap());
                    // log!("Moving pending node {} to active nodes", account_id);
                    false
                } else {
                    true
                }
            });
            log!("pending_nodes: {:?}", self.pending_nodes.to_vec());

            // TODO: replace after refactoring post_signed_batch
            let mut rng = rand::thread_rng();
            let random_number = rng.gen::<u64>();

            // if bootstrapping phase, wait until there are committee_size active nodes
            if self.bootstrapping_phase {
                if self.active_nodes.len() < self.config.committee_size {
                    log!("Not enough active nodes to exit bootstrapping phase");
                    return;
                }
                self.bootstrapping_phase = false;
                log!("Exiting bootstrapping phase");
                // select committees EPOCH_COMMITTEES_LOOKAHEAD epochs in advance
                for _ in 0..EPOCH_COMMITTEES_LOOKAHEAD {
                    let committee = self.select_committee(random_number);
                    self.committees.push(committee);
                }
            } else {
                // remove committee of last epoch
                self.committees.remove(0);
            }

            // select committee from active nodes
            let committee = self.select_committee(random_number);
            log!(
                "Selected committee for epoch {}: {:?}",
                self.get_current_epoch(),
                committee
            );
            self.committees.push(committee);

            // set last processed epoch to current epoch
            self.last_processed_epoch = self.get_current_epoch();
        });
    }
}
