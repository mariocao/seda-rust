use std::sync::Arc;

use seda_config::AnotherConfig;

use super::errors::Result;
use crate::ChainAdapterTrait;

#[derive(Debug)]
pub struct AnotherChain;

#[async_trait::async_trait]
impl ChainAdapterTrait for AnotherChain {
    type Client = Arc<()>;
    type Config = AnotherConfig;

    fn new_client(_config: &Self::Config) -> Result<Self::Client> {
        Ok(Arc::new(()))
    }

    async fn construct_signed_tx(
        _signer_account_id: Option<&str>,
        _signer_keypair: &[u8],
        _contract_id: &str,
        _method_name: &str,
        _args: Vec<u8>,
        _gas: u64,
        _deposit: u128,
        _server_url: &str,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }

    async fn construct_transfer_tx(
        _signer_account_id: Option<&str>,
        _signer_keypair: &[u8],
        _receiver_id: &str,
        _deposit: u128,
        _server_url: &str,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }

    async fn send_tx(_client: Self::Client, _signed_tx: &[u8]) -> Result<Vec<u8>> {
        unimplemented!()
    }

    async fn view(_client: Self::Client, _contract_id: &str, _method_name: &str, _args: Vec<u8>) -> Result<Vec<u8>> {
        unimplemented!()
    }
}
