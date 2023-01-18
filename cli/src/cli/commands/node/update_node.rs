use clap::{Args, Subcommand};
use seda_config::{ChainConfigs, NodeConfig, PartialDepositAndContractID};
use seda_runtime_sdk::Chain;
use serde::Serialize;
use serde_json::json;

use crate::{cli::commands::call, Result};

/// Update node commands
#[derive(Clone, Debug, Subcommand, Serialize)]
pub enum UpdateNodeCommand {
    AcceptOwnership,
    SetPendingOwner {
        #[arg(short, long)]
        owner: String,
    },
    SetSocketAddress {
        #[arg(short, long)]
        address: String,
    },
}

#[derive(Debug, Args)]
pub struct UpdateNode {
    #[arg(short, long)]
    pub node_id: u64,
    #[command(flatten)]
    pub details: PartialDepositAndContractID,
    #[command(subcommand)]
    pub command: UpdateNodeCommand,
}

impl UpdateNode {
    pub async fn handle(self, node_config: &NodeConfig, chains_config: &ChainConfigs) -> Result<()> {
        let args = json!({
                    "node_id": self.node_id.to_string(),
                    "command": self.command
        })
        .to_string();
        call::<String>(
            Chain::Near,
            &node_config.contract_account_id,
            "update_node",
            node_config.deposit,
            args,
            node_config,
            chains_config,
        )
        .await
    }
}
