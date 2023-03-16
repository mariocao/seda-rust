use clap::Args;
use seda_chains::{chain, Client};
use seda_config::{ChainConfigsInner, DelegateConfig};
use seda_runtime_sdk::Chain;

use crate::cli::{errors::Result, utils::to_yocto};

#[derive(Debug, Args)]
pub struct TopUp {
    /// The receiver account id you want to transfer to (ex. example.near)
    pub receiver: String,
    /// Amount of tokens to transfer in wholes (1 = 1 NEAR)
    pub amount:   u64,
}

impl TopUp {
    pub async fn handle(self, config: DelegateConfig) -> Result<()> {
        // Convert to yocto NEAR, which uses 24 decimals
        let amount_yocto = to_yocto(&self.amount.to_string());

        let signed_tx = chain::construct_transfer_tx(
            Chain::Near,
            &config.signer_account_id,
            &config.account_secret_key,
            &self.receiver,
            amount_yocto,
            &config.rpc_url,
        )
        .await?;

        let config = ChainConfigsInner::test_config();
        let client = Client::new(&Chain::Near, &config)?;

        println!("Sending {}N to {}..", self.amount, self.receiver);
        chain::send_tx(Chain::Near, client, &signed_tx).await?;
        println!("Transaction has been completed");

        Ok(())
    }
}
