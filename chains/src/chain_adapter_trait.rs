use std::{fmt::Debug, sync::Arc};

use jsonrpsee_types::Params;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{ChainAdapterError, Result};

pub struct TransactionParams {
    pub signer_acc_str: String,
    pub signer_sk_str:  String,
    pub contract_id:    String,
    pub method_name:    String,
    pub args:           Vec<u8>,
    pub gas:            u64,
    pub deposit:        u128,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "")]
pub struct NodeIds {
    pub contract_id: String,
    pub node_id:     u64,
}

#[derive(Serialize, Deserialize)]
pub struct NodeDetails {
    pub contract_id: String,
    pub limit:       u64,
    pub offset:      u64,
}

// TODO once rpc becomes a trait need to replace params type.
#[async_trait::async_trait]
pub trait ChainAdapterTrait: Debug + Send + Sync + 'static {
    /// The Config fields for the adapter specific implementation.
    type Config: Send + Sync;
    /// The Client type for the adapter specific implementation.
    type Client: Send + Sync + 'static;

    /// Returns an new instance of the client given the server address.
    fn new_client(config: &Self::Config) -> Result<Self::Client>;

    /// Returns a signed transaction given the necessary information.
    #[allow(clippy::too_many_arguments)]
    async fn construct_signed_tx(
        signer_acc_str: &str,
        signer_sk_str: &str,
        contract_id: &str,
        method_name: &str,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
        server_url: &str,
    ) -> Result<Vec<u8>>;
    /// Default trait function that calls sign and send specific
    /// implementations.
    async fn sign_and_send_tx(client: Arc<Self::Client>, tx_params: TransactionParams) -> Result<Vec<u8>> {
        let signed_tx = Self::sign_tx(client.clone(), tx_params).await?;
        Self::send_tx(client, &signed_tx).await
    }

    /// To sign a transaction for the adapter specific implementation.
    async fn sign_tx(client: Arc<Self::Client>, tx_params: TransactionParams) -> Result<Vec<u8>>;

    /// To send a transaction for the adapter specific implementation.
    async fn send_tx(client: Arc<Self::Client>, signed_tx: &[u8]) -> Result<Vec<u8>>;
    /// To view for the adapter specific implementation.
    async fn view(client: Arc<Self::Client>, contract_id: &str, method_name: &str, args: Vec<u8>) -> Result<Vec<u8>>;
    /// Default trait function to get the node owner.
    async fn get_node_owner(client: Arc<Self::Client>, params: Params<'_>) -> Result<Vec<u8>> {
        let method_name = "get_node_owner";
        let params = params
            .one::<NodeIds>()
            .map_err(|e| ChainAdapterError::BadParams(format!("{method_name}: {e}")))?;

        let args = json!({"node_id": params.node_id.to_string()}).to_string().into_bytes();

        Self::view(client, &params.contract_id.to_string(), method_name, args).await
    }

    /// Default trait function to get the node socket address.
    async fn get_node_socket_address(client: Arc<Self::Client>, params: Params<'_>) -> Result<Vec<u8>> {
        let method_name = "get_node_socket_address";
        let params = params
            .one::<NodeIds>()
            .map_err(|_| ChainAdapterError::BadParams(method_name.to_string()))?;

        let args = json!({"node_id": params.node_id.to_string()}).to_string().into_bytes();

        Self::view(client, &params.contract_id.to_string(), method_name, args).await
    }

    /// Default trait function to get the nodes.
    async fn get_nodes(client: Arc<Self::Client>, params: Params<'_>) -> Result<Vec<u8>> {
        let method_name = "get_nodes";

        let params = params
            .one::<NodeDetails>()
            .map_err(|_| ChainAdapterError::BadParams(method_name.to_string()))?;

        let args = json!({"limit": params.limit.to_string(), "offset": params.offset.to_string()})
            .to_string()
            .into_bytes();

        Self::view(client, &params.contract_id.to_string(), method_name, args).await
    }

    /// Default trait function to register the node.
    async fn register_node(client: Arc<Self::Client>, params: Params<'_>) -> Result<Vec<u8>> {
        let signed_tx = params
            .one::<Vec<u8>>()
            .map_err(|_| ChainAdapterError::BadParams("register_node".to_string()))?;

        Self::send_tx(client, &signed_tx).await
    }

    /// Default trait function to remove the node.
    async fn remove_node(client: Arc<Self::Client>, params: Params<'_>) -> Result<Vec<u8>> {
        let signed_tx = params
            .one::<Vec<u8>>()
            .map_err(|_| ChainAdapterError::BadParams("register_node".to_string()))?;

        Self::send_tx(client, &signed_tx).await
    }

    /// Default trait function to set the node socket address.
    async fn set_node_socket_address(client: Arc<Self::Client>, params: Params<'_>) -> Result<Vec<u8>> {
        let signed_tx = params
            .one::<Vec<u8>>()
            .map_err(|_| ChainAdapterError::BadParams("register_node".to_string()))?;

        Self::send_tx(client, &signed_tx).await
    }
}