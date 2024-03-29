use clap::Args;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};
use seda_config::AppConfig;

use crate::Result;

#[derive(Debug, Args)]
pub struct AddPeer {
    /// A libp2p compatible address (ex. /ip4/127.0.0.1/tcp/44635)
    pub multi_addr: String,
}

impl AddPeer {
    pub async fn handle(self, config: AppConfig) -> Result<()> {
        let client = WsClientBuilder::default()
            .build(format!("ws://{}", &config.seda_server_url))
            .await?;

        client.request("add_peer", rpc_params!(&self.multi_addr)).await?;
        println!("Peer {} has been added", &self.multi_addr);

        Ok(())
    }
}
