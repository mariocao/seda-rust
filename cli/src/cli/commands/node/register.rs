use clap::Args;
use seda_common::RegisterNodeArgs;
use seda_config::{AppConfig, PartialChainConfigs, PartialNodeConfig};
use seda_runtime_sdk::Chain;

use crate::{cli::commands::call, Result};

#[derive(Debug, Args)]
pub struct Register {
    #[arg(short, long)]
    pub register_deposit: u128,
    #[arg(short, long)]
    pub socket_address:   String,
    #[command(flatten)]
    pub node_config:      PartialNodeConfig,
}

impl Register {
    pub async fn handle(self, config: AppConfig, chains_config: PartialChainConfigs) -> Result<()> {
        let chains_config = config.chains.to_config(chains_config)?;

        let node_config = &config.node.to_config(self.node_config)?;
        let args = RegisterNodeArgs {
            multi_addr:       self.socket_address,
            // TODO: fix these once we merge in the KeyPair on node config changes.
            bn254_public_key: node_config.seda_key_pair.public_key.to_compressed().expect("TODO"),
            signature:        vec![0; 32],
        }
        .to_string();
        call::<String>(
            Chain::Near,
            &node_config.contract_account_id,
            "register_node",
            self.register_deposit,
            args,
            node_config,
            &chains_config,
        )
        .await
    }
}
