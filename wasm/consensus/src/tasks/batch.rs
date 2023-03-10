use clap::Args;
use seda_runtime_sdk::{
    log,
    wasm::{
        bn254_sign,
        call_self,
        chain_view,
        p2p_broadcast_message,
        shared_memory_get,
        shared_memory_set,
        Bn254PrivateKey,
        Promise,
        CONFIG,
    },
    Level,
    PromiseStatus,
};

#[derive(Debug, Args)]
pub struct Batch;

impl Batch {
    pub fn handle(self) {
        log!(Level::Debug, "Batch Handle");
        // TODO temp work around while env bug exists
        shared_memory_set(
            "private_key_bytes",
            CONFIG.seda_key_pair.private_key.to_bytes().expect("TODO"),
        );
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
    log!(Level::Debug, "Batch Step 1");
    // TODO temp work around while env bug exists
    let pk_bytes = shared_memory_get("private_key_bytes");
    let pk = Bn254PrivateKey::try_from(pk_bytes.as_slice()).expect("TODO");
    let result = Promise::result(0);
    match result {
        PromiseStatus::Fulfilled(Some(bytes)) => {
            log!(Level::Debug, "{bytes:?}");
            let result = bn254_sign(&bytes, &pk);
            p2p_broadcast_message(result.to_compressed().expect("TODO"))
                .start()
                .then(call_self("batch_step_2", vec![]));
        }
        other => log!(Level::Debug, "{other:?}"),
    };
}

#[no_mangle]
fn batch_step_2() {
    let result = Promise::result(0);
    log!(Level::Debug, "Batch Step 2 {result:?}");
    // TODO p2p_broadcast_message doesn't set this yet...
    // match result {
    //     PromiseStatus::Fulfilled(Some(bytes)) => {
    //         let str = String::from_bytes_vec(bytes).expect("TODO");
    //         log!(seda_runtime_sdk::Level::Debug, "Success: {str}");
    //     }
    //     other => log!(Level::Debug, "{other:?}"),
    // };
}
