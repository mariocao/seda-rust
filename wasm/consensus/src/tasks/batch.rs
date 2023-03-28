use std::str::FromStr;

use clap::Args;
use primitive_types::U256;
use seda_common::{ComputeMerkleRootResult, MainChainConfig};
use seda_runtime_sdk::{
    log,
    to_yocto,
    wasm::{
        bn254_sign,
        call_self,
        chain_call,
        chain_view,
        get_local_bn254_public_key,
        get_local_ed25519_public_key,
        get_oracle_contract_id,
        p2p_broadcast_message,
        shared_memory_get,
        shared_memory_set,
        Bn254PublicKey,
        Promise,
    },
    FromBytes,
    Level,
    PromiseStatus,
};
use serde_json::json;

use crate::{
    message::{BatchMessage, Message},
    types::batch_signature::{
        add_public_key,
        add_signature,
        get_or_create_batch_signature_store,
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
        shared_memory_set("contract_id", contract_id.clone().into());
        shared_memory_set(
            "ed25519_public_key",
            hex::decode(get_local_ed25519_public_key()).unwrap(),
        );

        chain_view(
            seda_runtime_sdk::Chain::Near,
            &contract_id,
            "compute_merkle_root",
            Vec::new(),
        )
        .start()
        .then(chain_view(
            seda_runtime_sdk::Chain::Near,
            &contract_id,
            "get_config",
            Vec::new(),
        ))
        .then(chain_view(
            seda_runtime_sdk::Chain::Near,
            contract_id,
            "get_last_generated_random_number",
            Vec::new(),
        ))
        .then(call_self("batch_step_1", vec![]));
    }
}

#[no_mangle]
fn batch_step_1() {
    log!(Level::Debug, "Batch Step 1");

    let bn254_public_key = shared_memory_get("bn254_public_key");
    let ed25519_public_key = shared_memory_get("ed25519_public_key");
    let contract_id = shared_memory_get("contract_id");
    let implicit_address = hex::encode(&ed25519_public_key);


    let result = Promise::result(0);
    let main_chain_config = if let PromiseStatus::Fulfilled(Some(config)) = Promise::result(1) {
        serde_json::from_slice::<MainChainConfig>(&config).expect("Config is not of type MainChainConfig")
    } else {
        panic!("Could not fetch config from contract");
    };

    let last_generated_number = if let PromiseStatus::Fulfilled(Some(num)) = Promise::result(2) {
        let encoded = serde_json::from_slice::<String>(&num).expect("random number is not a string");
        U256::from_str(&encoded).expect("Generated number is not a U256")
    } else {
        panic!("Could not fetch random number");
    };

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
            let mut signature_store = get_or_create_batch_signature_store(
                BATCH_SIGNATURE_STORE_KEY,
                Some(batch.current_slot),
                Some(batch.clone().merkle_root),
            );

            let current_slot_leader = batch.clone().current_slot_leader.unwrap_or("00000".to_string());

            if current_slot_leader == implicit_address {
                log!(Level::Debug, "Wowow we are the leader");

                if main_chain_config.committee_size == signature_store.signatures.len() as u64 {
                    let random_number_bytes: Vec<u8> = if last_generated_number.is_zero() {
                        vec![0]
                    } else {
                        let mut random_number_slice: Vec<u8> = Vec::new();
                        last_generated_number.to_little_endian(&mut random_number_slice);
                        random_number_slice
                    };

                    let leader_signature = bn254_sign(&random_number_bytes);

                    chain_call(
                        seda_runtime_sdk::Chain::Near,
                        String::from_utf8(contract_id).unwrap(),
                        "post_signed_batch",
                        json!({
                            "aggregate_signature": signature_store.aggregated_signature,
                            "aggregate_public_key": signature_store.aggregated_public_keys,
                            "signers": signature_store.signers,
                            "leader_signature": leader_signature.to_compressed().unwrap()
                        })
                        .to_string()
                        .into_bytes(),
                        to_yocto("0.001"),
                    )
                    .start();
                    log!(Level::Debug, "WOOOOOOW SUBMITTTTTTT BITCHESSSSS");
                } else {
                    log!(
                        Level::Debug,
                        "We have to wait a lil, config:{} and signatures: {}",
                        main_chain_config.committee_size,
                        signature_store.signatures.len()
                    );
                }
            }

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
                batch:              batch.merkle_root,
                bn254_public_key:   bn254_public_key.clone(),
                signature:          signature.to_compressed().expect("TODO"),
                ed25519_public_key: ed25519_public_key.clone(),
            });

            signature_store.aggregated_signature = add_signature(signature_store.aggregated_signature, signature)
                .to_compressed()
                .expect("Could not compress Bn254 signature");

            signature_store.signers.push(hex::encode(&ed25519_public_key));

            signature_store.aggregated_public_keys = add_public_key(
                signature_store.aggregated_public_keys,
                Bn254PublicKey::from_compressed(&bn254_public_key).expect("Could not derive key"),
            )
            .to_compressed()
            .expect("Could not compress Bn254 Public Key");

            signature_store
                .signatures
                .insert(hex::encode(&bn254_public_key), signature.to_compressed().unwrap());

            signature_store.slot = batch.current_slot;

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
