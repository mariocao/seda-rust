use clap::Args;
use seda_common::{GetNodeArgs, NodeInfo};
use seda_config::{AppConfig, PartialChainConfigs};
use seda_runtime_sdk::Chain;

use crate::{cli::commands::view, Result};

#[derive(Debug, Args)]
pub struct Node {
    #[arg(short, long)]
    pub contract_id:       Option<String>,
    #[arg(short, long)]
    pub signer_account_id: Option<String>,
}

impl Node {
    pub async fn handle(self, config: AppConfig, chains_config: PartialChainConfigs) -> Result<()> {
        let chains_config = config.chains.to_config(chains_config)?;

        let (signer_account_id, contract_account_id) = config.node.ids(self.signer_account_id, self.contract_id)?;
        let args = GetNodeArgs::from(signer_account_id).to_string();
        view::<Option<NodeInfo>>(
            Chain::Near,
            &contract_account_id,
            "get_node",
            Some(args),
            &chains_config,
        )
        .await
    }
}
