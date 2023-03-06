use clap::Args;
use jsonrpsee::server::ServerBuilder;
use tracing::debug;

use super::test_rpc::{MockNearRpc, MockNearRpcServer};
use crate::Result;

#[derive(Debug, Args)]
pub struct Start;

impl Start {
    #[tokio::main]
    pub async fn handle(self, addr: &str) -> Result<()> {
        let server = ServerBuilder::default().build(addr).await?;

        debug!("Starting Seda Test RPC server on {addr}");
        let handle = server.start(MockNearRpc.into_rpc())?;
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("failed to listen for event");
            debug!("Shutting down debug RPC");
            handle.stopped().await
        })
        .await
        .expect("FOO");
        Ok(())
    }
}
