use clap::Args;
use seda_common::{GetNodeArgs, NodeInfo};
use seda_config::{AppConfig, PartialChainConfigs};
use seda_runtime_sdk::Chain;

use crate::{cli::commands::view, Result};

/// Returns node information for a given implicit account id, incl. balance and
/// registered public keys.
#[derive(Debug, Args)]
pub struct Node {
    /// SEDA contract account id
    #[arg(short, long)]
    pub contract_id: Option<String>,
    /// Node implicit account id (Ed25519 public key in hex)
    #[arg(short, long)]
    pub node_id:     Option<String>,
}

impl Node {
    pub async fn handle(self, config: AppConfig, chains_config: PartialChainConfigs) -> Result<()> {
        let chains_config = config.chains.to_config(chains_config)?;

        let contract_id = if let Some(contract_id) = self.contract_id {
            contract_id
        } else {
            config
                .node
                .contract_account_id
                .clone()
                .expect("contract_id is not configured")
        };

        let node_id = if let Some(node_id) = self.node_id {
            node_id
        } else {
            hex::encode(config.node.get_node_public_key()?)
        };

        let args = GetNodeArgs::from(node_id).to_string();
        view::<Option<NodeInfo>>(Chain::Near, &contract_id, "get_node", Some(args), &chains_config).await
    }
}
