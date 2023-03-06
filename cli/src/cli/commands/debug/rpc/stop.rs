use clap::Args;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};

use crate::Result;

#[derive(Debug, Args)]
pub struct Stop;

impl Stop {
    #[tokio::main]
    pub async fn handle(self, addr: &str) -> Result<()> {
        let client = WsClientBuilder::default().build(format!("ws://{}", addr)).await?;
        assert!(client.request::<(), _>("stop_server", rpc_params!()).await.is_err());
        Ok(())
    }
}
