use clap::Args;
use jsonrpsee::{core::client::ClientT, http_client::HttpClientBuilder, rpc_params};

use crate::Result;

#[derive(Debug, Args)]
pub struct Stop;

impl Stop {
    #[tokio::main]
    pub async fn handle(self, addr: &str) -> Result<()> {
        let client = HttpClientBuilder::default().build(format!("http://{addr}"))?;
        client.request::<(), _>("stop_server", rpc_params!()).await?;
        Ok(())
    }
}
