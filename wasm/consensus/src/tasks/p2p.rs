use std::str::FromStr;

use clap::Args;
use seda_runtime_sdk::{
    log,
    wasm::{bn254_verify, shared_memory_set, Bn254PublicKey, Bn254Signature},
    Level,
};

use crate::{
    message::Message,
    types::batch_signature::{
        add_public_key,
        add_signature,
        get_or_create_batch_signature_store,
        BATCH_SIGNATURE_STORE_KEY,
    },
};

#[derive(Debug, Args)]
pub struct P2P {
    // TODO should change to bytes for more efficiency
    message: String,
}

impl P2P {
    /// This function should behave as follows:
    ///  1. Generic verifications:
    ///     * Check if batch is valid
    ///         - signed by a member of current epoch committee
    ///         - valid bn254 signature
    ///     * Otherwise, it is ignored and P2P counter-measure is triggered
    ///  2. Check batch header (i.e., merkle root)
    ///     * If current known batch, append to list of accumulated signatures
    ///     * If unknown batch, add to future batch list (current batch was
    ///       submitted)
    pub fn handle(self) {
        let message = Message::from_str(&self.message).expect("Failed to decode message");

        match message {
            Message::Batch(batch_message) => {
                // Step 1: batch verifications
                // TODO: check that batch was signed by a member of the epoch committee

                // Check valid bn254 signature
                let bn254_signature = Bn254Signature::from_compressed(&batch_message.signature)
                    .expect("Could not get signature from compressed bytes");
                let bn254_public_key = Bn254PublicKey::from_compressed(&batch_message.bn254_public_key)
                    .expect("Could not get signature from compressed bytes");

                if !bn254_verify(
                    &batch_message.batch_header,
                    &bn254_signature,
                    &Bn254PublicKey::from_compressed(&batch_message.bn254_public_key).unwrap(),
                ) {
                    // TODO: Check if we should disconnect p2p node/slashed/measures
                    log!(
                        Level::Warn,
                        "[P2PTask] Received P2P batch message with an invalid Bn254 signature"
                    );

                    return;
                }

                // Step 2: check batch message data
                let mut signature_store = get_or_create_batch_signature_store(BATCH_SIGNATURE_STORE_KEY);

                // Case 1: batch message for same batch header / merkle root
                if batch_message.batch_header == signature_store.batch_header {
                    // Check if batch signature was already been included
                    let bn254_public_key_str = hex::encode(&batch_message.bn254_public_key);
                    if signature_store.signatures.contains_key(&bn254_public_key_str) {
                        // TODO: Check if we should disconnect p2p node/slashed/measures
                        log!(
                            Level::Warn,
                            "[P2PTask] Received P2P batch message with duplicated signature"
                        );
                        
                        return;
                    }

                    // Aggregate signature and public key
                    let new_aggregate_signature = add_signature(signature_store.aggregated_signature, bn254_signature)
                        .to_compressed()
                        .expect("Could not compress Bn254 signature");
                    let new_aggregate_public_key =
                        add_public_key(signature_store.aggregated_public_keys, bn254_public_key)
                            .to_compressed()
                            .expect("Could not compress Bn254 Public Key");
                    let ed25519_public_key_str = hex::encode(&batch_message.ed25519_public_key);

                    signature_store.aggregated_signature = new_aggregate_signature;
                    signature_store.aggregated_public_keys = new_aggregate_public_key;
                    signature_store.signers.push(ed25519_public_key_str);
                    signature_store
                        .signatures
                        .insert(bn254_public_key_str, batch_message.signature);

                    log!(
                        Level::Debug,
                        "[P2PTask] Added new signature to batch #{} (total: {})",
                        hex::encode(&signature_store.batch_header),
                        signature_store.signatures.len()
                    );
                }
                // Case 2: batch message for unknown batch header / merkle root
                else {
                    log!(
                        Level::Warn,
                        "[P2PTask] Received a P2P batch message for a (yet) unknown batch header"
                    );
                    // TODO: accumulate future batch messages to be processed by Batch Task
                    todo!("Not yet implemented because batch header is always the same");
                }

                // Save changes in shared memory
                shared_memory_set(
                    BATCH_SIGNATURE_STORE_KEY,
                    serde_json::to_string(&signature_store).unwrap().into(),
                );
            }
        }
    }
}
