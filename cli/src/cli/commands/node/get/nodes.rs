use clap::Args;
use seda_common::{GetNodesArgs, NodeInfo};
use seda_config::{AppConfig, PartialChainConfigs, PartialNodeConfig};
use seda_runtime_sdk::Chain;

use crate::{cli::commands::view, Result};

/// Returns a list of node information, incl. balance and registered public
/// keys.
#[derive(Debug, Args)]
pub struct Nodes {
    /// Number of items to be returned
    #[arg(short, long, default_value_t = 10)]
    pub limit:       u64,
    /// Number of items to be ommitted in the returned list
    #[arg(short, long, default_value_t = 0)]
    pub offset:      u64,
    /// SEDA contract account id
    #[arg(short, long)]
    pub contract_id: Option<String>,
}

impl Nodes {
    pub async fn handle(self, config: AppConfig, chains_config: PartialChainConfigs) -> Result<()> {
        let chains_config = config.chains.to_config(chains_config)?;
        let node_config = config
            .node
            .to_config(PartialNodeConfig::default())
            .expect("Could not get default node configuration");

        let contract_id = self.contract_id.unwrap_or(node_config.contract_account_id.clone());

        let args = GetNodesArgs::from((self.limit, self.offset)).to_string();
        view::<Vec<NodeInfo>>(Chain::Near, &contract_id, "get_nodes", Some(args), &chains_config).await
    }
}
