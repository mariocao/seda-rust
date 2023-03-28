use bn254::PublicKey;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    log,
    near_bindgen,
    AccountId,
};
use sha2::{Digest, Sha256};

use crate::{manage_storage_deposit, merkle::CryptoHash, MainchainContract, MainchainContractExt};

pub type BatchHeight = u64;
pub type BatchId = CryptoHash;

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct BatchHeader {
    pub height:     BatchHeight,
    pub state_root: CryptoHash,
}

/// Batch data without merkle roots (stored on contract)
#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Batch {
    pub header:       BatchHeader,
    pub transactions: Vec<String>,
}

/// Batch data using merkle roots (used to calculate batch id)
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MerklizedBatch {
    pub prev_root:    BatchId,
    pub header:       BatchHeader,
    pub transactions: Vec<u8>,
}

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    pub fn get_latest_batch_id(&self) -> BatchId {
        self.batch_ids_by_height.get(&self.num_batches).unwrap_or_default()
    }

    #[payable]
    pub fn post_signed_batch(
        &mut self,
        // Bn254 aggregate signature
        aggregate_signature: Vec<u8>,
        // Bn254 aggregate public key
        aggregate_public_key: Vec<u8>,
        // Ed25519 public keys of bn254 signers
        signers: Vec<AccountId>,
        // Ed25519 signature
        leader_signature: Vec<u8>,
    ) {
        manage_storage_deposit!(self, {
            // update the epoch if necessary
            self.process_epoch();

            log!("slot leader: {}", self.get_current_slot_leader().unwrap());
            log!("block: {}", env::block_height());
            assert_eq!(self.get_current_slot_leader().unwrap(), env::signer_account_id());
            let leader_pk = PublicKey::from_compressed(
                self.active_nodes
                    .get(&env::signer_account_id())
                    .unwrap()
                    .bn254_public_key,
            )
            .unwrap()
            .to_compressed()
            .unwrap();
            assert!(
                self.bn254_verify(
                    self.last_generated_random_number.to_le_bytes().to_vec(),
                    leader_signature.clone(),
                    leader_pk
                ),
                "Invalid slot leader signature"
            );
            let current_slot = self.get_current_slot();
            let hash = Sha256::digest([leader_signature, current_slot.to_le_bytes().to_vec()].concat());
            let new_random: near_bigint::U256 = near_bigint::U256::from_little_endian(&hash);
            self.last_generated_random_number = new_random;

            // require the data request accumulator to be non-empty
            assert!(
                !self.data_request_accumulator.is_empty(),
                "Data request accumulator is empty"
            );

            // reconstruct the aggregate public key from signers[] to verify all signers are
            // eligible for this batch while also verifying individual eligibility
            let current_committee = self.committees.get(&self.get_current_epoch()).unwrap();
            assert!(
                current_committee.contains(&signers[0]),
                "Node is not part of the committee"
            );
            let aggregate_public_key_reconstructed = signers.iter().skip(1).fold(
                PublicKey::from_compressed(self.active_nodes.get(&signers[0]).unwrap().bn254_public_key).unwrap(),
                |acc, signer| {
                    assert!(current_committee.contains(signer), "Node is not part of the committee");
                    let signer_public_key =
                        PublicKey::from_compressed(self.active_nodes.get(signer).unwrap().bn254_public_key).unwrap();
                    acc + signer_public_key
                },
            );
            assert!(
                aggregate_public_key_reconstructed.to_compressed().unwrap() == aggregate_public_key,
                "Invalid aggregate public key"
            );

            // verify aggregate signature
            let merkle_root = self.internal_compute_merkle_root();
            assert!(
                self.bn254_verify(merkle_root.clone(), aggregate_signature, aggregate_public_key),
                "Invalid aggregate signature"
            );

            let header = BatchHeader {
                height:     self.num_batches + 1,
                state_root: CryptoHash::default(), // TODO
            };

            // create batch
            let batch = Batch {
                header:       header.clone(),
                transactions: self.data_request_accumulator.to_vec(),
            };

            // calculate batch id
            let batch_id = CryptoHash::hash_borsh(&MerklizedBatch {
                prev_root: self.get_latest_batch_id(),
                header,
                transactions: merkle_root,
            });

            // store batch
            self.num_batches += 1;
            self.batch_by_id.insert(&batch_id, &batch);
            self.batch_ids_by_height.insert(&self.num_batches, &batch_id);

            // clear data request accumulator
            self.data_request_accumulator.clear();
        }); // end manage_storage_deposit
    }
}
