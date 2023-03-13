use getrandom::{register_custom_getrandom, Error};
use near_sdk::{env, near_bindgen, AccountId};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::{MainchainContract, MainchainContractExt};
register_custom_getrandom!(get_random_in_near);

/// Contract private methods
impl MainchainContract {
    pub fn select_committee(&mut self) -> Vec<AccountId> {
        let mut rng = rand::thread_rng();
        let random_number = rng.gen::<u64>();

        let validators = self.active_nodes.keys_as_vector().to_vec();
        let mut chosen_committee: Vec<AccountId> = Vec::new();
        let mut chosen_indices: Vec<u128> = Vec::new();
        let mut rerolls = 0;
        for i in 0..self.config.committee_size {
            let mut hasher = Sha256::new();

            let input = [random_number.to_le_bytes(), u64::try_from(i).unwrap().to_le_bytes()].concat();

            // write input message
            hasher.update(input);

            // read hash digest and consume hasher
            let prn = hasher.finalize();
            let prn_bytes: [u8; 16] = prn[0..16].try_into().expect("slice with incorrect length");
            let mut chosen_index = u128::from_le_bytes(prn_bytes)
                % u128::try_from(validators.len()).expect("Couldn't fetch validators len");

            while chosen_indices.contains(&chosen_index) {
                rerolls = rerolls + 1;

                let mut hasher = Sha256::new();
                let input = [
                    random_number.to_le_bytes(),
                    u64::try_from(i + self.config.committee_size + rerolls)
                        .unwrap()
                        .to_le_bytes(),
                ]
                .concat();
                // write input message
                hasher.update(input);
                // read hash digest and consume hasher
                let prn = hasher.finalize();
                let prn_bytes: [u8; 16] = prn[0..16].try_into().expect("slice with incorrect length");
                chosen_index = u128::from_le_bytes(prn_bytes)
                    % u128::try_from(validators.len()).expect("Couldn't fetch validators len");
            }

            chosen_indices.push(chosen_index);
            let validator = validators
                .get(usize::try_from(chosen_index).unwrap())
                .expect("couldn't fetch validator");
            chosen_committee.push(validator.clone());
        }
        chosen_committee
    }
}

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    pub fn get_committees(&self) -> Vec<Vec<AccountId>> {
        self.committees.clone()
    }
}

pub fn get_random_in_near(buf: &mut [u8]) -> Result<(), Error> {
    let random = env::random_seed();
    buf.copy_from_slice(&random);
    Ok(())
}
