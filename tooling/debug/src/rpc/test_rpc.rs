use std::str::FromStr;

use async_trait::async_trait;
use jsonrpsee::{core::Error, proc_macros::rpc};
use near_crypto::{PublicKey, Signature};
use near_jsonrpc_client::methods::query::RpcQueryResponse;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::{
    account::{AccessKey, AccessKeyPermission},
    borsh::BorshDeserialize,
    hash::CryptoHash,
    merkle::{Direction, MerklePathItem},
    transaction::SignedTransaction,
    views::{
        CallResult,
        ExecutionMetadataView,
        ExecutionOutcomeView,
        ExecutionOutcomeWithIdView,
        FinalExecutionOutcomeView,
        SignedTransactionView,
    },
};
use rand::{rngs::OsRng, Rng};
use serde_json::json;
use tokio::sync::mpsc::Sender;

use crate::Result;

#[rpc(server)]
pub trait MockNearRpc {
    #[method(name = "broadcast_tx_async")]
    async fn broadcast_tx_async(&self, args: String) -> Result<CryptoHash, Error>;

    #[method(name = "query")]
    async fn query(
        &self,
        account_id: String,
        args_base64: Option<String>,
        finality: String,
        method_name: Option<String>,
        request_type: String,
    ) -> Result<RpcQueryResponse, Error>;

    #[method(name = "stop_server")]
    async fn stop_server(&self) -> Result<(), Error>;

    #[method(name = "tx")]
    async fn tx(&self, hash: String, account_id: String) -> Result<FinalExecutionOutcomeView, Error>;
}

pub struct MockNearRpc {
    shutdown_channel: Sender<bool>,
}

impl MockNearRpc {
    pub fn new(shutdown_channel: Sender<bool>) -> Self {
        Self { shutdown_channel }
    }

    fn random_crypto_hash(&self) -> CryptoHash {
        CryptoHash::hash_bytes(&OsRng.gen::<[u8; 32]>())
    }
}

#[async_trait]
impl MockNearRpcServer for MockNearRpc {
    async fn broadcast_tx_async(&self, params: String) -> Result<CryptoHash, Error> {
        println!("Calling broadcast_tx_async");
        let non_base_64 = near_primitives::serialize::from_base64(&params).unwrap();

        let tx: SignedTransaction =
            SignedTransaction::try_from_slice(&non_base_64).map_err(|e| Error::Custom(e.to_string()))?;
        Ok(tx.get_hash())
    }

    // TODO: would be nice to set up logging so we can confirm things from node side
    // as well
    async fn query(
        &self,
        _account_id: String,
        _args_base64: Option<String>,
        _finality: String,
        method_name: Option<String>,
        request_type: String,
    ) -> Result<RpcQueryResponse, Error> {
        match request_type.as_str() {
            "call_function" if method_name.is_some() => match method_name.unwrap().as_str() {
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
                    block_hash:   self.random_crypto_hash(),
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
                    block_hash:   self.random_crypto_hash(),
                }),
                "compute_merkle_root" => Ok(RpcQueryResponse {
                    kind:         QueryResponseKind::CallResult(CallResult {
                        // TODO: this needs to be a valid random hash :)
                        // This can be done in the batch sign pr.
                        result: serde_json::to_vec_pretty(&json!(self.random_crypto_hash().as_bytes())).unwrap(),
                        logs:   Default::default(),
                    }),
                    block_height: 119467302,
                    block_hash:   self.random_crypto_hash(),
                }),
                _ => unimplemented!(),
            },
            "view_access_key" => Ok(RpcQueryResponse {
                kind:         QueryResponseKind::AccessKey(
                    AccessKey {
                        nonce:      100288680000299,
                        permission: AccessKeyPermission::FullAccess,
                    }
                    .into(),
                ),
                block_height: 119467302,
                block_hash:   self.random_crypto_hash(),
            }),
            _ => unimplemented!(),
        }
    }

    async fn stop_server(&self) -> Result<(), Error> {
        println!("Shutting down Seda Test RPC");
        self.shutdown_channel
            .send(true)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        Ok(())
    }

    async fn tx(&self, _hash: String, _account_id: String) -> Result<FinalExecutionOutcomeView, Error> {
        println!("calling tx");
        // TODO this would normally look up the tx but for now we have dummy data
        // let tx_info = TransactionInfo::TransactionId {
        //     hash:       CryptoHash::from_str(&hash).unwrap(),
        //     account_id: account_id.parse().unwrap(),
        // };

        Ok(FinalExecutionOutcomeView {
            status:              near_primitives::views::FinalExecutionStatus::SuccessValue(vec![8]),
            transaction:         SignedTransactionView {
                signer_id:   "seda-debug.near".parse().unwrap(),
                public_key:  PublicKey::from_str("ed25519:Eyyt8zB1NfpQcYGSXLRg83Xx7xk6Z2ohPfVhouuyYY1Y").unwrap(),
                nonce:       100288680000344,
                receiver_id: "mc.seda-debug.near".parse().unwrap(),
                actions:     vec![],
                signature:   Signature::default(),
                hash:        self.random_crypto_hash(),
            },
            transaction_outcome: ExecutionOutcomeWithIdView {
                proof:      vec![MerklePathItem {
                    hash:      self.random_crypto_hash(),
                    direction: Direction::Right,
                }],
                block_hash: self.random_crypto_hash(),
                id:         self.random_crypto_hash(),
                outcome:    ExecutionOutcomeView {
                    logs:         Vec::new(),
                    receipt_ids:  vec![self.random_crypto_hash()],
                    gas_burnt:    2428030560766,
                    tokens_burnt: 242803056076600000000,
                    executor_id:  "seda-debug.near".parse().unwrap(),
                    status:       near_primitives::views::ExecutionStatusView::SuccessReceiptId(
                        self.random_crypto_hash(),
                    ),
                    metadata:     ExecutionMetadataView {
                        version:     1,
                        gas_profile: None,
                    },
                },
            },
            receipts_outcome:    Vec::new(),
        })
    }
}
