mod start;
mod stop;
mod test_rpc;
use clap::Subcommand;

use crate::Result;

#[derive(Debug, Subcommand)]
pub enum Rpc {
    /// Starts the test RPC server
    Start(start::Start),
    /// Stops the test RPC server
    Stop(stop::Stop),
}

impl Rpc {
    pub fn handle(self, addr: &str) -> Result<()> {
        match self {
            Rpc::Start(start) => start.handle(addr),
            Rpc::Stop(stop) => stop.handle(addr),
        }
    }
}
