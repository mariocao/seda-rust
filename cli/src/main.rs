mod cli;
use clap::Parser;
use cli::CliOptions;
mod errors;
use errors::Result;

fn main() -> Result<()> {
    // Load the dotenv file first since our config overloads values from it.
    dotenv::dotenv().ok();
    // Parse the CLI Options.
    let options = CliOptions::parse();
    // Load the config before starting our logger.
    let (config, partial_log_config) = seda_config::create_and_load_or_load_config(None);
    // We hold the guards so logging works properly.
    let _guard = seda_logger::init(&partial_log_config.to_config(options.log_options.clone())?);
    options.command.handle(config)
}
