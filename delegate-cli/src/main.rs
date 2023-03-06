use clap::Parser;
use seda_config::{Config, PartialDelegateConfig};

use crate::cli::CliOptions;

mod cli;

fn main() {
    // Load the dotenv file first since our config overloads values from it.
    dotenv::dotenv().ok();

    let options = CliOptions::parse();
    let mut env_config = PartialDelegateConfig::default();
    env_config.overwrite_from_env();

    let config = options.delegate_config.to_config(env_config).unwrap();

    options.command.handle(config);
}
