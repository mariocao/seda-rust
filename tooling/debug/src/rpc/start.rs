use clap::Args;
use jsonrpsee::server::ServerBuilder;
use tokio::sync::mpsc;

use super::test_rpc::{MockNearRpc, MockNearRpcServer};
use crate::Result;

#[derive(Debug, Args)]
pub struct Start;

impl Start {
    #[tokio::main]
    pub async fn handle(self, addr: &str) -> Result<()> {
        let server = ServerBuilder::default().build(addr).await?;

        println!("Starting Seda Test RPC server on {addr}");
        let (tx, mut rx) = mpsc::channel(1);
        let rpc = MockNearRpc::new(tx);
        let handle = server.start(rpc.into_rpc())?;
        let spawn_handle = handle.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("failed to listen for event");
            println!("Shutting down Seda Test RPC");
            spawn_handle.stop().expect("TODO");
        });

        while let Some(shutdown) = rx.recv().await {
            if shutdown {
                handle.stop()?;
            }
        }
        Ok(())
    }
}
