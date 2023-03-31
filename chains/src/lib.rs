//! Defines a Chain type based on features when compiling.

mod another_chain;
pub use another_chain::AnotherChain;
use seda_config::ChainConfigs;
use seda_runtime_sdk::Chain;

mod errors;

pub use errors::*;

mod chain_adapter_trait;
pub use chain_adapter_trait::*;

mod near_chain;
pub use near_chain::NearChain;

#[derive(Debug, Clone)]
pub enum Client {
    Another(<AnotherChain as ChainAdapterTrait>::Client),
    Near(<NearChain as ChainAdapterTrait>::Client),
}

impl Client {
    pub fn new(chain: &Chain, chains_config: &ChainConfigs) -> Result<Self> {
        Ok(match chain {
            Chain::Another => Self::Another(AnotherChain::new_client(&chains_config.another)?),
            Chain::Near => Self::Near(NearChain::new_client(&chains_config.near)?),
        })
    }

    fn another(&self) -> <AnotherChain as ChainAdapterTrait>::Client {
        if let Self::Another(v) = self {
            v.clone()
        } else {
            unreachable!()
        }
    }

    fn near(&self) -> <NearChain as ChainAdapterTrait>::Client {
        if let Self::Near(v) = self {
            v.clone()
        } else {
            unreachable!()
        }
    }
}

pub mod chain {
    use super::*;
    #[allow(clippy::too_many_arguments)]
    pub async fn construct_signed_tx(
        chain: Chain,
        signer_account_id: Option<&str>,
        signer_keypair: &[u8],
        contract_id: &str,
        method_name: &str,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
        server_url: &str,
    ) -> Result<Vec<u8>> {
        match chain {
            Chain::Another => {
                AnotherChain::construct_signed_tx(
                    signer_account_id,
                    signer_keypair,
                    contract_id,
                    method_name,
                    args,
                    gas,
                    deposit,
                    server_url,
                )
                .await
            }
            Chain::Near => {
                NearChain::construct_signed_tx(
                    signer_account_id,
                    signer_keypair,
                    contract_id,
                    method_name,
                    args,
                    gas,
                    deposit,
                    server_url,
                )
                .await
            }
        }
    }

    pub async fn construct_transfer_tx(
        chain: Chain,
        signer_account_id: Option<&str>,
        signer_keypair: &[u8],
        receiver_id: &str,
        amount: u128,
        server_url: &str,
    ) -> Result<Vec<u8>> {
        match chain {
            Chain::Another => {
                AnotherChain::construct_transfer_tx(signer_account_id, signer_keypair, receiver_id, amount, server_url)
                    .await
            }
            Chain::Near => {
                NearChain::construct_transfer_tx(signer_account_id, signer_keypair, receiver_id, amount, server_url)
                    .await
            }
        }
    }

    pub async fn send_tx(chain: Chain, client: Client, signed_tx: &[u8]) -> Result<Vec<u8>> {
        match chain {
            Chain::Another => AnotherChain::send_tx(client.another(), signed_tx).await,
            Chain::Near => NearChain::send_tx(client.near(), signed_tx).await,
        }
    }

    pub async fn view(
        chain: Chain,
        client: Client,
        contract_id: &str,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>> {
        match chain {
            Chain::Another => AnotherChain::view(client.another(), contract_id, method_name, args).await,
            Chain::Near => NearChain::view(client.near(), contract_id, method_name, args).await,
        }
    }
}
