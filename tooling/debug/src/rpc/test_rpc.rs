use async_trait::async_trait;
use jsonrpsee::{core::Error, proc_macros::rpc};
use near_jsonrpc_client::methods::query::RpcQueryResponse;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::{hash::CryptoHash, views::CallResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc::Sender;

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

    #[method(name = "query")]
    async fn query(
        &self,
        account_id: String,
        args_base64: String,
        finality: String,
        method_name: String,
        request_type: String,
    ) -> Result<RpcQueryResponse, Error>;

    #[method(name = "stop_server")]
    async fn stop_server(&self) -> Result<(), Error>;
}

pub struct MockNearRpc {
    shutdown_channel: Sender<bool>,
}

impl MockNearRpc {
    pub fn new(shutdown_channel: Sender<bool>) -> Self {
        Self { shutdown_channel }
    }
}

#[async_trait]
impl MockNearRpcServer for MockNearRpc {
    async fn compute_merkle_root(&self, _: Vec<String>) -> Result<Batch, Error> {
        println!("Calling compute_merkle_root");
        Ok(Batch::dummy())
    }

    async fn query(
        &self,
        _account_id: String,
        _args_base64: String,
        _finality: String,
        method_name: String,
        _request_type: String,
    ) -> Result<RpcQueryResponse, Error> {
        match method_name.as_str() {
            "get_node" => Ok(RpcQueryResponse {
                kind:         QueryResponseKind::CallResult(CallResult {
                    // TODO we have this structure defined already in the CLI
                    // So we can move it to somewhere common
                    result: serde_json::to_vec_pretty(&json!({
                            "owner":          "near_rpc_mocked",
                            "pending_owner":  None::<String>,
                            "socket_address": "127.0.0.1:6666"
                    }))
                    .unwrap(),
                    logs:   Default::default(),
                }),
                block_height: 119467302,
                block_hash:   CryptoHash::new(),
            }),
            "get_nodes" => Ok(RpcQueryResponse {
                kind:         QueryResponseKind::CallResult(CallResult {
                    // TODO we have this structure defined already in the CLI
                    // So we can move it to somewhere common
                    result: serde_json::to_vec_pretty(&json!([{
                                    "owner":          "near_rpc_mocked",
                                    "pending_owner":  None::<String>,
                                    "socket_address": "127.0.0.1:6666"
                    }]))
                    .unwrap(),
                    logs:   Default::default(),
                }),
                block_height: 119467302,
                block_hash:   CryptoHash::new(),
            }),
            _ => unimplemented!(),
        }
    }

    async fn stop_server(&self) -> Result<(), Error> {
        println!("Shutting down Seda Test RPC");
        self.shutdown_channel.send(true).await.expect("failed to send");
        Ok(())
    }
}
