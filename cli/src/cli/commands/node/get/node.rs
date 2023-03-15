use clap::Args;
use seda_common::{GetNodeArgs, HumanReadableNode};
use seda_config::{AppConfig, PartialChainConfigs};
use seda_runtime_sdk::Chain;

use crate::{cli::commands::view, Result};

#[derive(Debug, Args)]
pub struct Node {
    #[arg(short, long)]
    pub contract_id: Option<String>,
}

impl Node {
    pub async fn handle(self, config: AppConfig, chains_config: PartialChainConfigs) -> Result<()> {
        let chains_config = config.chains.to_config(chains_config)?;

        let contract_account_id = config.node.to_contract_account_id(self.contract_id)?;
        let args = GetNodeArgs::from(contract_account_id.clone()).to_string();
        view::<Option<HumanReadableNode>>(Chain::Near, &contract_account_id, "get_node", args, &chains_config).await
    }
}
