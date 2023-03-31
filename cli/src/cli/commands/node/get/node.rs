use clap::Args;
use seda_common::{GetNodeArgs, NodeInfo};
use seda_config::{AppConfig, PartialChainConfigs, PartialNodeConfig};
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
        let node_config = config
            .node
            .to_config(PartialNodeConfig::default())
            .expect("Could not get default node configuration");

        let contract_id = self.contract_id.unwrap_or(node_config.contract_account_id.clone());
        let node_id = self
            .node_id
            .unwrap_or(hex::encode(node_config.keypair_ed25519.public_key.to_bytes()));

        let args = GetNodeArgs::from(node_id).to_string();

        view::<Option<NodeInfo>>(Chain::Near, &contract_id, "get_node", Some(args), &chains_config).await
    }
}
