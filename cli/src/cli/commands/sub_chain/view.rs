use clap::Args;
use seda_config::{AppConfig, PartialChainConfigs};
use seda_runtime_sdk::Chain;

use crate::{cli::commands::view, Result};

#[derive(Debug, Args)]
pub struct View {
    /// The sub-chain to call.
    chain:       Chain,
    /// The contract ID for the sub-chain.
    contract_id: String,
    /// The method name to view.
    method_name: String,
    /// The args to pass to the view method.
    args:        Option<String>,
}

impl View {
    pub async fn handle(self, config: AppConfig, chains_config: PartialChainConfigs) -> Result<()> {
        let chains_config = config.chains.to_config(chains_config)?;
        view::<serde_json::Value>(
            self.chain,
            &self.contract_id,
            &self.method_name,
            self.args,
            &chains_config,
        )
        .await
    }
}
