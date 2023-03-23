use std::collections::HashMap;

use seda_runtime_sdk::{
    wasm::{shared_memory_contains_key, shared_memory_get, Bn254Signature},
    FromBytes,
};
use serde::{Deserialize, Serialize};

pub const BATCH_SIGNATURE_STORE_KEY: &str = "batch_signatures";

#[derive(Serialize, Deserialize)]
pub struct BatchSignatureStore {
    pub aggregated_signature: Vec<u8>,
    pub signatures:           HashMap<String, Vec<u8>>,
}

pub fn get_or_create_batch_signature_store(storage_key: &str) -> BatchSignatureStore {
    if shared_memory_contains_key(storage_key) {
        let result = shared_memory_get(&storage_key);
        let json_str = String::from_bytes_vec(result).unwrap();

        serde_json::from_str(&json_str).unwrap()
    } else {
        return BatchSignatureStore {
            aggregated_signature: Vec::new(),
            signatures:           HashMap::new(),
        };
    }
}

pub fn add_signature(aggregated_signature: Vec<u8>, signature: Bn254Signature) -> Bn254Signature {
    if aggregated_signature.is_empty() {
        return signature;
    }

    Bn254Signature::from_compressed(aggregated_signature).expect("Given Bn254Signature signature is not decodable")
        + signature
}
