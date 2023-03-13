use clap::Args;
use seda_runtime_sdk::wasm::p2p_broadcast_message;

#[derive(Debug, Args)]
pub struct P2p {
    message: String,
}

impl P2p {
    pub fn handle(self) {
        p2p_broadcast_message(self.message.into_bytes()).start();
    }
}
