use clap::Args;
use seda_chains::{chain, Client};
use seda_config::{ChainConfigsInner, DelegateConfig};
use seda_runtime_sdk::Chain;

use crate::cli::utils::to_yocto;

#[derive(Debug, Args)]
pub struct TopUp {
    /// The receiver account id you want to transfer to (ex. example.near)
    pub receiver: String,
    /// Amount of tokens to transfer in wholes (1 = 1 NEAR)
    pub amount:   String,
}

impl TopUp {
    pub async fn handle(self, config: DelegateConfig) {
        // Convert to yocto NEAR, which uses 24 decimals
        let amount_yocto = to_yocto(&self.amount);

        let signed_tx = chain::construct_transfer_tx(
            Chain::Near,
            &config.signer_account_id,
            &config.account_secret_key,
            &self.receiver,
            amount_yocto,
            &config.rpc_url,
        )
        .await
        .unwrap();

        let config = ChainConfigsInner::test_config();
        let client = Client::new(&Chain::Near, &config).unwrap();

        println!("Sending {}N to {}..", self.amount, self.receiver);
        chain::send_tx(Chain::Near, client, &signed_tx).await.unwrap();
        println!("Transaction has been completed");
    }
}
