use clap::Args;
use seda_common::UpdateNode;
use seda_config::{AppConfig, PartialChainConfigs, PartialNodeConfig};
use seda_runtime_sdk::Chain;

use crate::{cli::commands::call, Result};

#[derive(Debug, Args)]
pub struct Update {
    #[command(flatten)]
    pub node_config: PartialNodeConfig,
    #[command(subcommand)]
    pub command:     UpdateNode,
}

impl Update {
    pub async fn handle(self, config: AppConfig, chains_config: PartialChainConfigs) -> Result<()> {
        let chains_config = config.chains.to_config(chains_config)?;
        let node_config = &config.node.to_config(self.node_config)?;

        let args = self.command.to_string();
        call::<Option<serde_json::Value>>(
            Chain::Near,
            &node_config.contract_account_id,
            "update_node",
            0,
            args,
            node_config,
            &chains_config,
        )
        .await
    }
}
