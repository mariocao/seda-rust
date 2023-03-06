mod rpc;
use clap::Subcommand;
pub use rpc::*;
use seda_config::AppConfig;

use crate::Result;

#[derive(Debug, Subcommand)]
pub enum DebugMode {
    /// The test RPC for testing.
    #[command(subcommand)]
    TestRpc(Rpc),
}

impl DebugMode {
    pub fn handle(self, config: AppConfig) -> Result<()> {
        match self {
            DebugMode::TestRpc(rpc) => rpc.handle(&config.seda_server_url),
        }
    }
}
