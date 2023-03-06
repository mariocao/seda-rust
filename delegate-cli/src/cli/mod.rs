use clap::{command, Parser, Subcommand};
use seda_config::{DelegateConfig, PartialDelegateConfig};
mod commands;

mod utils;

#[derive(Parser)]
#[command(name = "seda-delegate")]
#[command(author = "https://github.com/SedaProtocol")]
#[command(version)]
#[command(propagate_version = true)]
#[command(about = "For staking & delegation on the SEDA protocol.", long_about = None)]
#[command(next_line_help = true)]
pub struct CliOptions {
    #[command(flatten)]
    pub delegate_config: PartialDelegateConfig,
    #[command(subcommand)]
    pub command:         Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Top-up a target account by a given amount in NEAR
    TopUp(commands::top_up::TopUp),
}

impl Command {
    #[tokio::main]
    pub async fn handle(self, config: DelegateConfig) {
        match self {
            Self::TopUp(top_up) => {
                top_up.handle(config).await;
            }
        }
    }
}
