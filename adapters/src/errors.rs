use near_jsonrpc_client::methods::broadcast_tx_async::RpcBroadcastTxAsyncError;
use near_primitives::account::id::ParseAccountError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum NearAdapterError {
    #[error("error calling contract change method")]
    CallChangeMethod(String),

    #[error("error calling contract view method")]
    CallViewMethod,

    #[error("time limit exceeded for the transaction to be recognized")]
    BadTransactionTimestamp,

    #[error("could not deserialize status to string")]
    BadDeserialization(#[from] serde_json::Error),

    #[error("missing parameter: `{0}` is not set")]
    MissingParam(String),

    #[error("error parsing string to near secretkey")]
    ParseAccountId(#[from] ParseAccountError),

    #[error("near json rpc query error")]
    JsonRpcQueryError(
        #[from] near_jsonrpc_client::errors::JsonRpcError<near_jsonrpc_client::methods::query::RpcQueryError>,
    ),

    #[error("near json rpc tx error")]
    JsonRpcTxError(#[from] near_jsonrpc_client::errors::JsonRpcError<RpcBroadcastTxAsyncError>),
}

pub type Result<T, E = NearAdapterError> = core::result::Result<T, E>;