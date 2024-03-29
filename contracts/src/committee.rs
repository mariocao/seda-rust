use getrandom::{register_custom_getrandom, Error};
use near_sdk::{env, near_bindgen, AccountId};
use sha2::{Digest, Sha256};

use crate::{MainchainContract, MainchainContractExt};
register_custom_getrandom!(get_random_in_near);

/// Contract private methods
impl MainchainContract {
    pub fn select_committee(&mut self, random_number: near_bigint::U256) -> Vec<AccountId> {
        let validators = self.active_nodes.keys_as_vector().to_vec();
        let mut chosen_committee: Vec<AccountId> = Vec::new();
        let mut chosen_indices: Vec<usize> = Vec::new();
        let mut rerolls = 0;
        // choose `config.committee_size` validators from current active nodes
        for i in 0..self.config.committee_size {
            let hash = Sha256::digest([random_number.to_le_bytes().as_ref(), i.to_le_bytes().as_ref()].concat());
            let prn: near_bigint::U256 = near_bigint::U256::from_little_endian(&hash);
            let mut chosen_index: usize = (prn % validators.len()).as_usize();

            // if the chosen validator index was previously selected, we fetch another one
            while chosen_indices.contains(&chosen_index) {
                rerolls += 1;

                let hash = Sha256::digest(
                    [
                        random_number.to_le_bytes().as_ref(),
                        (i + self.config.committee_size + rerolls).to_le_bytes().as_ref(),
                    ]
                    .concat(),
                );
                let prn: near_bigint::U256 = near_bigint::U256::from_little_endian(&hash);
                chosen_index = (prn % validators.len()).as_usize();
            }
            chosen_indices.push(chosen_index);
            let validator = validators.get(chosen_index).expect("couldn't fetch validator");
            chosen_committee.push(validator.clone());
        }

        chosen_committee
    }
}

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    pub fn get_committee(&self, epoch: u64) -> Option<Vec<AccountId>> {
        self.committees.get(&epoch)
    }

    pub fn get_last_generated_random_number(&self) -> near_bigint::U256 {
        self.last_generated_random_number
    }

    pub fn get_current_slot_leader(&self) -> Option<AccountId> {
        let current_committee = self.committees.get(&self.get_current_epoch())?;
        let hash = Sha256::digest(
            [
                self.last_generated_random_number.to_le_bytes().as_ref(),
                self.get_current_slot().to_le_bytes().as_ref(),
            ]
            .concat(),
        );
        let prn: near_bigint::U256 = near_bigint::U256::from_little_endian(&hash);
        let chosen_index = (prn % self.config.committee_size).as_usize();
        current_committee.get(chosen_index).cloned()
    }
}

pub fn get_random_in_near(buf: &mut [u8]) -> Result<(), Error> {
    let random = env::random_seed();
    buf.copy_from_slice(&random);
    Ok(())
}
