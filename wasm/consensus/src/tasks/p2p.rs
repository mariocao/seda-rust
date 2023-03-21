use clap::Args;
use seda_runtime_sdk::{
    log,
    p2p::MessageKind,
    wasm::{bn254_verify, p2p_broadcast_message, shared_memory_get, Bn254Signature, CONFIG},
    Level,
};

#[derive(Debug, Args)]
pub struct P2P {
    message: String,
    kind:    MessageKind,
}

impl P2P {
    pub fn handle(self) {
        match self.kind {
            MessageKind::Batch => {
                let batch_bytes = shared_memory_get("latest_batch");
                let signature_bytes = hex::decode(self.message.clone()).expect("TODO");
                let signature = Bn254Signature::from_compressed(signature_bytes).expect("TODO");
                let verified = bn254_verify(&batch_bytes, &signature, &CONFIG.seda_key_pair.public_key);
                log!(Level::Debug, "Verified: {verified}");
                if verified {
                    // Then we send the message to others? Maybe this order should be the other way
                    // around for speed?
                    // Or actually should this be a success message?
                    p2p_broadcast_message(self.message.into_bytes(), self.kind).start();
                }
            }
        }
    }
}
