use bn254::PublicKey;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    near_bindgen,
    AccountId,
};

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
        aggregate_signature: Vec<u8>,
        aggregate_public_key: Vec<u8>,
        signers: Vec<AccountId>,
    ) {
        // require the data request accumulator to be non-empty
        assert!(
            !self.data_request_accumulator.is_empty(),
            "Data request accumulator is empty"
        );

        // reconstruct the aggregate public key from signers[] to verify all signers are
        // eligible for this batch while also verifying individual eligibility

        // 1. initialize with the first signer
        assert!(self.active_nodes.get(&signers[0]).is_some(), "Node is not active");
        let mut aggregate_public_key_check =
            PublicKey::from_compressed(self.active_nodes.get(&signers[0]).unwrap().bn254_public_key).unwrap();
        // 2. add the rest of the signers' public keys
        for signer in signers.iter().skip(1) {
            assert!(self.active_nodes.get(signer).is_some(), "Node is not active");
            let signer_public_key =
                PublicKey::from_compressed(self.active_nodes.get(signer).unwrap().bn254_public_key).unwrap();
            aggregate_public_key_check = aggregate_public_key_check + signer_public_key;
        }
        assert!(
            aggregate_public_key_check.to_compressed().unwrap() == aggregate_public_key,
            "Invalid aggregate public key"
        );

        // verify aggregate signature
        let merkle_root = self.compute_merkle_root();
        assert!(
            self.bn254_verify(merkle_root, aggregate_signature, aggregate_public_key),
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
            transactions: self.compute_merkle_root(),
        });

        manage_storage_deposit!(self, "require", {
            // store batch
            self.num_batches += 1;
            self.batch_by_id.insert(&batch_id, &batch);
            self.batch_ids_by_height.insert(&self.num_batches, &batch_id);

            // clear data request accumulator
            self.data_request_accumulator.clear();
        }); // end manage_storage_deposit
    }
}
