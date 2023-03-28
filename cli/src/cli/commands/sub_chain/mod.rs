use clap::Subcommand;
use seda_config::{AppConfig, PartialChainConfigs};

use crate::Result;

mod call;
mod view;

#[derive(Debug, Subcommand)]
pub enum SubChain {
    // seda sub-chain call near mc.mennat0.testnet register_node "{\"socket_address\":\"127.0.0.1:8080\"}"
    // 870000000000000000000
    /// Calls the specified method on the specified chain with the given args
    /// and contract ID.
    Call(Box<call::Call>),
    // TODO maybe a ListMethods command,
    // seda sub-chain view near mc.mennat0.testnet get_active_nodes "{\"offset\":\"0\",\"limit\":\"2\"}"
    /// Views the specified method on the specified chain with the given args
    /// and contract ID.
    View(view::View),
}

impl SubChain {
    #[tokio::main]
    pub async fn handle(self, config: AppConfig, chains_config: PartialChainConfigs) -> Result<()> {
        match self {
            Self::Call(call) => call.handle(config, chains_config).await,
            Self::View(view) => view.handle(config, chains_config).await,
        }
    }
}
