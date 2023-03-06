use async_trait::async_trait;
use jsonrpsee::{core::Error, proc_macros::rpc};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::Result;

// TODO move to common shared module between contracts and rest of seda
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Batch {
    pub block_hash:   String,
    pub block_height: usize,
    pub logs:         Vec<String>,
    pub result:       Vec<u8>,
}

impl Batch {
    pub fn dummy() -> Self {
        Self {
            block_hash: "CJCJu5syUJAvd4hpZTvkKXCLL7AorQKqbzG1VVPHmjPx".to_string(),
            block_height: 118965159,
            result: vec![
                156, 191, 146, 9, 129, 121, 82, 79, 233, 74, 115, 19, 126, 55, 218, 230, 227, 226, 234, 223, 176, 245,
                34, 101, 245, 155, 196, 69, 19, 245, 153, 244,
            ],
            ..Default::default()
        }
    }
}

#[rpc(server)]
pub trait MockNearRpc {
    #[method(name = "compute_merkle_root")]
    async fn compute_merkle_root(&self, args: Vec<String>) -> Result<Batch, Error>;

    #[method(name = "stop_server")]
    async fn stop_server(&self) -> Result<(), Error>;
}

pub struct MockNearRpc;

#[async_trait]
impl MockNearRpcServer for MockNearRpc {
    async fn compute_merkle_root(&self, _: Vec<String>) -> Result<Batch, Error> {
        debug!("compute merkle root");
        Ok(Batch::dummy())
    }

    async fn stop_server(&self) -> Result<(), Error> {
        debug!("stopping debug RPC server");
        self.stop_server().await
    }
}
