use std::collections::HashMap;

use clap::Args;
use seda_common::ComputeMerkleRootResult;
use seda_runtime_sdk::{
    log,
    wasm::{
        bn254_sign,
        call_self,
        chain_view,
        get_local_bn254_public_key,
        get_oracle_contract_id,
        p2p_broadcast_message,
        shared_memory_contains_key,
        shared_memory_get,
        shared_memory_set,
        Promise,
    },
    Bytes,
    FromBytes,
    Level,
    PromiseStatus,
    ToBytes,
};

use crate::{
    message::{BatchMessage, Message},
    types::batch_signature::{
        add_signature,
        get_or_create_batch_signature_store,
        BatchSignatureStore,
        BATCH_SIGNATURE_STORE_KEY,
    },
};

#[derive(Debug, Args)]
pub struct Batch;

impl Batch {
    pub fn handle(self) {
        let contract_id = get_oracle_contract_id();
        log!(Level::Debug, "Batch Handle {contract_id}");

        // TODO: Temp fix, need to fix env variables
        shared_memory_set("bn254_public_key", hex::decode(get_local_bn254_public_key()).unwrap());

        chain_view(
            seda_runtime_sdk::Chain::Near,
            contract_id,
            "compute_merkle_root",
            Vec::new(),
        )
        .start()
        .then(call_self("batch_step_1", vec![]));
    }
}

#[no_mangle]
fn batch_step_1() {
    log!(Level::Debug, "Batch Step 1");

    let bn254_public_key = shared_memory_get("bn254_public_key");
    let result = Promise::result(0);

    // let compute_merk

    // sign message
    // Batch should point to previous batch
    // after validation accumulate the signatures
    // check if slot leader
    // post it

    match result {
        PromiseStatus::Fulfilled(Some(batch_bytes)) => {
            // TODO: Unwraps
            let batch_string = String::from_bytes_vec(batch_bytes).unwrap();
            let batch: ComputeMerkleRootResult = serde_json::from_str(&batch_string).unwrap();

            log!(Level::Debug, "Batch: {:?}", &batch);

            let signature = bn254_sign(&batch.merkle_root);

            log!(
                Level::Debug,
                "Sending batch {} with bn254pk: {}",
                hex::encode(&batch.merkle_root),
                hex::encode(&bn254_public_key)
            );

            // TODO: Verify that this batch points to the previous batch
            let message = Message::Batch(BatchMessage {
                batch:            batch.merkle_root,
                bn254_public_key: bn254_public_key.clone(),
                signature:        signature.to_compressed().expect("TODO"),
            });

            let mut signature_store = get_or_create_batch_signature_store(BATCH_SIGNATURE_STORE_KEY);

            signature_store.aggregated_signature = add_signature(signature_store.aggregated_signature, signature)
                .to_compressed()
                .expect("Could not compress Bn254 signature");

            signature_store
                .signatures
                .insert(hex::encode(&bn254_public_key), signature.to_compressed().unwrap());

            shared_memory_set(
                BATCH_SIGNATURE_STORE_KEY,
                serde_json::to_string(&signature_store).unwrap().into(),
            );

            p2p_broadcast_message(serde_json::to_vec(&message).unwrap()).start();
        }
        PromiseStatus::Rejected(error) => {
            let err = String::from_bytes_vec(error).unwrap();
            log!(Level::Error, "{err:?}");
        }

        other => {
            log!(Level::Debug, "{other:?}");
        }
    };
}
