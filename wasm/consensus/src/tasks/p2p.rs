use std::str::FromStr;

use clap::Args;
use seda_runtime_sdk::{
    log,
    wasm::{bn254_verify, shared_memory_get, Bn254Signature, CONFIG},
    Level,
};

use crate::message::{Message, MessageKind};

#[derive(Debug, Args)]
pub struct P2P {
    // TODO should change to bytes for more efficiency
    message: String,
}

impl P2P {
    pub fn handle(self) {
        log!(Level::Debug, "P2P Handle {self:?}");
        let message = Message::from_str(&self.message).expect("Failed to decode message");
        match message.kind {
            MessageKind::Batch => {
                let batch_bytes = shared_memory_get("latest_batch");
                let signature_bytes = hex::decode(self.message).expect("Failed to hex decode message");
                let signature = Bn254Signature::from_compressed(signature_bytes).expect("failed to get signature");
                let verified = bn254_verify(&batch_bytes, &signature, &CONFIG.seda_key_pair.public_key);
                log!(Level::Debug, "Verified: {verified}");
                if verified {
                    // TODO: what to do on success.
                }
            }
        }
    }
}
