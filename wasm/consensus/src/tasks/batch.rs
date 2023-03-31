use clap::Args;
use primitive_types::U256;
use seda_common::{ComputeMerkleRootResult, MainChainConfig};
use seda_runtime_sdk::{
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
    *,
};
use serde_json::json;

use crate::{
    message::{BatchMessage, Message},
    types::batch_signature::{
        add_public_key,
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
        log!(Level::Debug, "[BatchTask] Starting task for contract id: {contract_id}");

        // TODO: Temp fix, need to fix env variables
        shared_memory_set("contract_id", contract_id.clone().into());
        shared_memory_set(
            "ed25519_public_key",
            hex::decode(get_local_ed25519_public_key()).expect("Missing WASI env var for ED25519 public key"),
        );
        shared_memory_set(
            "bn254_public_key",
            hex::decode(get_local_bn254_public_key()).expect("Missing WASI env var for BN254 public key"),
        );

        chain_view(
            seda_runtime_sdk::Chain::Near,
            &contract_id,
            "compute_merkle_root",
            Vec::new(),
        )
        .start()
        // TODO: config logic should be moved to its own task
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
    // Retrieve data from shared memory
    let contract_id = shared_memory_get("contract_id");
    let bn254_public_key = shared_memory_get("bn254_public_key");
    let ed25519_public_key = shared_memory_get("ed25519_public_key");
    let mut signature_store = get_or_create_batch_signature_store(BATCH_SIGNATURE_STORE_KEY);

    // Retrieve results from promise results
    let batch: ComputeMerkleRootResult = match Promise::result(0) {
        PromiseStatus::Fulfilled(Some(batch_bytes)) => serde_json::from_slice(&batch_bytes)
            .expect("Cannot convert `merkle_root` json to `ComputeMerkleRootResult`"),
        PromiseStatus::Rejected(error) => {
            let err = String::from_bytes_vec(error).unwrap();
            panic!("`compute_merkle_root` promise rejected: {err:?}");
        }
        other => {
            panic!("`compute_merkle_root` promise other: {other:?}");
        }
    };
    let chain_config = if let PromiseStatus::Fulfilled(Some(config)) = Promise::result(1) {
        serde_json::from_slice::<MainChainConfig>(&config).expect("Config is not of type `MainChainConfig`")
    } else {
        panic!("Could not fetch config from contract");
    };
    let last_random_number = if let PromiseStatus::Fulfilled(Some(num)) = Promise::result(2) {
        // Example of encoded number:
        // 85808566236214186893554888775712866405891396064732569795826684455150103772489
        let encoded = serde_json::from_slice::<String>(&num).expect("random number is not a string");
        U256::from_dec_str(&encoded).expect("Generated number is not a U256")
    } else {
        panic!("Could not fetch random number");
    };

    let node_implicit_account = hex::encode(&ed25519_public_key);
    log!(
        Level::Debug,
        "[BatchTask][Slot #{}] Processing batch #{} (leader: {})",
        &batch.current_slot,
        hex::encode(&batch.merkle_root),
        Some(&node_implicit_account) == batch.current_slot_leader.as_ref()
    );

    // ==========================
    // || Process BATCH Logic  ||
    // ==========================

    // Case 1. Check if it was already processed
    if batch.merkle_root == signature_store.batch_header && batch.current_slot == signature_store.slot {
        log!(
            Level::Debug,
            "[BatchTask][Slot #{}] Ignoring batch #{} (already processed and recently broadcasted)",
            batch.current_slot,
            hex::encode(&batch.merkle_root)
        );
    }
    // Case 2. Check if was processed but not broadcasted during this slot
    else if batch.merkle_root == signature_store.batch_header && batch.current_slot != signature_store.slot {
        log!(
            Level::Debug,
            "[BatchTask][Slot #{}] Broadcasting previous batch #{} (already processed)",
            batch.current_slot,
            hex::encode(&batch.merkle_root)
        );

        signature_store.slot = batch.current_slot;
        shared_memory_set(
            BATCH_SIGNATURE_STORE_KEY,
            serde_json::to_string(&signature_store).unwrap().into(),
        );

        p2p_broadcast_message(signature_store.p2p_message.clone()).start();
    }
    // Case 3. Process new batch with different merkle root
    else {
        log!(
            Level::Debug,
            "[BatchTask][Slot #{}] Processing new batch #{}",
            batch.current_slot,
            hex::encode(&batch.merkle_root)
        );

        // FIXME: Verify that this batch points to the previous batch
        let bn254_signature = bn254_sign(&batch.merkle_root);

        // Update signature store with new batch data
        signature_store = BatchSignatureStore::new(batch.current_slot, batch.clone().merkle_root);

        signature_store.aggregated_signature = add_signature(signature_store.aggregated_signature, bn254_signature)
            .to_compressed()
            .expect("Could not compress Bn254 signature");

        signature_store.aggregated_public_keys = add_public_key(
            signature_store.aggregated_public_keys,
            Bn254PublicKey::from_compressed(&bn254_public_key).expect("Could not derive key"),
        )
        .to_compressed()
        .expect("Could not compress Bn254 Public Key");

        signature_store.signers.push(hex::encode(&ed25519_public_key));

        signature_store
            .signatures
            .insert(hex::encode(&bn254_public_key), bn254_signature.to_compressed().unwrap());

        signature_store.slot = batch.current_slot;

        let message = Message::Batch(BatchMessage {
            batch_header: batch.clone().merkle_root,
            bn254_public_key,
            signature: bn254_signature.to_compressed().expect("TODO"),
            ed25519_public_key,
        });
        signature_store.p2p_message =
            serde_json::to_vec(&message).expect("`BatchMessage` could not be serialized to bytes");

        // TODO: process accumulated batch messages from previous P2P tasks

        shared_memory_set(
            BATCH_SIGNATURE_STORE_KEY,
            serde_json::to_string(&signature_store)
                .expect("Could not convert SignatureStore to json")
                .into(),
        );

        p2p_broadcast_message(signature_store.p2p_message.clone()).start();
    }

    // ========================
    // || SLOT LEADER Logic  ||
    // ========================

    let current_slot_leader = match batch.current_slot_leader {
        Some(leader) => leader,
        None => panic!("Main-chain contract still bootstrapping (no slot leader)"),
    };

    // 1. Check if node is currrent leader
    if current_slot_leader == node_implicit_account {
        log!(
            Level::Info,
            "[BatchTask][Slot #{}] Selected as slot leader (got {}/{} signatures for batch #{})",
            batch.current_slot,
            signature_store.signatures.len(),
            chain_config.committee_size,
            hex::encode(&batch.merkle_root)
        );
        // 2. Check if node has stored all signatures
        // TODO: Change to 2/3 in the future
        if chain_config.committee_size == signature_store.signatures.len() as u64 {
            let mut last_random_value_bytes: [u8; 32] = [0; 32];
            last_random_number.to_little_endian(&mut last_random_value_bytes);

            let leader_signature_bytes = bn254_sign(&last_random_value_bytes)
                .to_compressed()
                .expect("Could not compress Bn254 signaturre");

            let contract_id_string =
                String::from_utf8(contract_id).expect("Could not read contract id from shared memory");

            log!(
                Level::Info,
                "[BatchTask][Slot #{}] Submitting signed batch #{} to contract `{}` with {}/{} aggregated signagutes",
                batch.current_slot,
                hex::encode(&batch.merkle_root),
                contract_id_string,
                signature_store.signatures.len(),
                chain_config.committee_size,
            );

            chain_call(
                seda_runtime_sdk::Chain::Near,
                contract_id_string,
                "post_signed_batch",
                json!({
                    "aggregate_signature": signature_store.aggregated_signature,
                    "aggregate_public_key": signature_store.aggregated_public_keys,
                    "signers": signature_store.signers,
                    "leader_signature": leader_signature_bytes
                })
                .to_string()
                .into_bytes(),
                // TODO: double-check deposit value
                to_yocto("1"),
            )
            .start();
        }
    }
}
