use std::str::FromStr;

use clap::Args;
use seda_runtime_sdk::{
    log,
    wasm::{bn254_verify, shared_memory_get, shared_memory_set, Bn254PublicKey, Bn254Signature},
    Level,
};

use crate::{
    message::Message,
    types::batch_signature::{add_signature, get_or_create_batch_signature_store, BATCH_SIGNATURE_STORE_KEY},
};

#[derive(Debug, Args)]
pub struct P2P {
    // TODO should change to bytes for more efficiency
    message: String,
}

impl P2P {
    pub fn handle(self) {
        let message = Message::from_str(&self.message).expect("Failed to decode message");

        match message {
            Message::Batch(batch_message) => {
                // let batch_bytes = shared_memory_get("latest_batch");
                // let signature_bytes = hex::decode(self.message).expect("Failed to hex decode
                // message");
                let signature =
                    Bn254Signature::from_compressed(batch_message.signature).expect("failed to get signature");

                // TODO: Also validate if the batch contents are from the current batch
                let verified = bn254_verify(
                    &batch_message.batch,
                    &signature,
                    &Bn254PublicKey::from_compressed(&batch_message.bn254_public_key).unwrap(),
                );

                if !verified {
                    // TODO: Check if we should disconnect p2p node/slashed/measures
                    log!(Level::Warn, "Bn254Signature is not valid");
                    return;
                }

                let mut signature_store = get_or_create_batch_signature_store(BATCH_SIGNATURE_STORE_KEY);

                signature_store.aggregated_signature = add_signature(signature_store.aggregated_signature, signature)
                    .to_compressed()
                    .expect("Could not compress Bn254 signature");

                signature_store.signatures.insert(
                    hex::encode(&batch_message.bn254_public_key),
                    signature.to_compressed().unwrap(),
                );

                log!(
                    Level::Debug,
                    "Added new signature, got ({}) signatures",
                    signature_store.signatures.len()
                );

                shared_memory_set(
                    BATCH_SIGNATURE_STORE_KEY,
                    serde_json::to_string(&signature_store).unwrap().into(),
                );

                // let verified = bn254_verify(&batch_bytes, &signature,
                // &CONFIG.seda_key_pair.public_key);
                // if verified {
                // TODO: what to do on success.
                // }
            }
        }
    }
}
