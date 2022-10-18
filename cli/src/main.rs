use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "seda")]
#[command(author = "https://github.com/SedaProtocol")]
#[command(version = "0.1.0")]
#[command(about = "For interacting with the seda protocol.", long_about = None)]
struct Options {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Sends a JsonRPC message to the node's server.
    Register,
    /// Runs the SEDA node
    Run,
}

fn main() {
    let options = Options::parse();

    if let Some(command) = options.command {
        match command {
            Commands::Register => todo!(),
            Commands::Run => seda_node::run(),
        }
    } else {
        todo!()
    }
}