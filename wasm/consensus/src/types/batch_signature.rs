use std::collections::HashMap;

use seda_runtime_sdk::{
    wasm::{shared_memory_contains_key, shared_memory_get, Bn254PublicKey, Bn254Signature},
    FromBytes,
};
use serde::{Deserialize, Serialize};

pub const BATCH_SIGNATURE_STORE_KEY: &str = "batch_signatures";

#[derive(Serialize, Deserialize)]
pub struct BatchSignatureStore {
    pub aggregated_signature:   Vec<u8>,
    pub aggregated_public_keys: Vec<u8>,
    /// Vec of accountIds (implicit ed25519 public keys)
    pub signers:                Vec<String>,
    pub signatures:             HashMap<String, Vec<u8>>,
    pub slot:                   u64,
    pub root:                   Vec<u8>,
}

pub fn get_or_create_batch_signature_store(
    storage_key: &str,
    slot: Option<u64>,
    root: Option<Vec<u8>>,
) -> BatchSignatureStore {
    if shared_memory_contains_key(storage_key) {
        let result = shared_memory_get(storage_key);
        let json_str = String::from_bytes_vec(result).unwrap();

        serde_json::from_str(&json_str).unwrap()
    } else {
        BatchSignatureStore {
            aggregated_signature:   Vec::new(),
            aggregated_public_keys: Vec::new(),
            signatures:             HashMap::new(),
            slot:                   slot.unwrap_or(0),
            root:                   root.unwrap_or(Vec::new()),
            signers:                Vec::new(),
        }
    }
}

pub fn add_signature(aggregated_signature: Vec<u8>, signature: Bn254Signature) -> Bn254Signature {
    if aggregated_signature.is_empty() {
        return signature;
    }

    Bn254Signature::from_compressed(aggregated_signature).expect("Given Bn254Signature signature is not decodable")
        + signature
}

pub fn add_public_key(aggregated_public_key: Vec<u8>, public_key: Bn254PublicKey) -> Bn254PublicKey {
    if aggregated_public_key.is_empty() {
        return public_key;
    }

    Bn254PublicKey::from_compressed(aggregated_public_key).expect("Given Bn254PublicKey is not decodable") + public_key
}
