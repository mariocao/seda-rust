use clap::Args;
use seda_runtime_sdk::{
    log,
    wasm::{bn254_sign, call_self, chain_view, p2p_broadcast_message, Bn254PrivateKey, Promise, CONFIG},
    FromBytes,
    Level,
    PromiseStatus,
};

#[derive(Debug, Args)]
pub struct Batch;

impl Batch {
    pub fn handle(self) {
        log!(Level::Debug, "Batch Handle");
        chain_view(
            seda_runtime_sdk::Chain::Near,
            &CONFIG.contract_account_id,
            "compute_merkle_root",
            Vec::new(),
        )
        .start()
        .then(call_self("batch_step_1", vec![]));
    }
}

#[no_mangle]
fn batch_step_1() {
    log!(Level::Debug, "Batch Step 1, {:?}", &*CONFIG);
    let result = Promise::result(0);
    match result {
        PromiseStatus::Fulfilled(Some(bytes)) => {
            log!(Level::Debug, "{bytes:?}");
            let result = bn254_sign(&bytes, &CONFIG.seda_key_pair.private_key);
            p2p_broadcast_message(result.to_compressed().expect("TODO"))
                .start()
                .then(call_self("batch_step_2", vec![]));
        }
        _ => todo!(),
    };
}

#[no_mangle]
fn batch_step_2() {
    log!(Level::Debug, "Batch Step 2");
    let result = Promise::result(0);
    match result {
        PromiseStatus::Fulfilled(Some(bytes)) => {
            let str = String::from_bytes_vec(bytes).expect("TODO");
            log!(seda_runtime_sdk::Level::Debug, "Success: {str}");
        }
        _ => todo!(),
    };
}
