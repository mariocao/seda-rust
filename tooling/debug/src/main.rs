use anyhow::Result;

mod rpc;
use clap::Parser;
pub use rpc::*;

#[derive(Debug, Parser)]
#[command(name = "seda_debug")]
#[command(author = "https://github.com/SedaProtocol")]
#[command(version)]
#[command(propagate_version = true)]
#[command(about = "For help debugging the SEDA protocol.", long_about = None)]
#[command(next_line_help = true)]
/// For debugging and testing the seda node and tools
pub enum DebugMode {
    /// The test RPC for testing.
    TestRpc {
        #[arg(short, long)]
        addr: String,
        #[command(subcommand)]
        rpc:  Rpc,
    },
}

impl DebugMode {
    pub fn handle(self) -> Result<()> {
        match self {
            DebugMode::TestRpc { addr, rpc } => rpc.handle(&addr),
        }
    }
}

fn main() -> Result<()> {
    let options = DebugMode::parse();
    options.handle()
}
