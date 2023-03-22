use clap::Args;
use seda_chains::{chain, Client};
use seda_config::{ChainConfigsInner, DelegateConfig};
use seda_crypto::MasterKey;
use seda_runtime_sdk::Chain;
use serde_json::json;

use crate::cli::{errors::Result, utils::to_yocto};

#[derive(Debug, Args)]
pub struct Stake {
    /// The contract address to stake on
    pub delegation_contract_id: String,
    /// The amount of SEDA tokens to stake (1 = 1 SEDA)
    pub amount:                 u64,
}

impl Stake {
    pub async fn handle(self, config: DelegateConfig) -> Result<()> {
        // SEDA tokens are in the same denominator as NEAR (24 decimals)
        let amount_yocto = to_yocto(&self.amount.to_string());
        let validator_master_key = MasterKey::try_from(&config.validator_master_key)?;
        let ed25519_key = validator_master_key.derive_ed25519(0)?;
        let ed25519_public_key = ed25519_key.public_key.as_bytes();

        let account_id = hex::encode(ed25519_public_key);

        println!(
            "Staking {} SEDA on {} for node {account_id}..",
            &self.amount, self.delegation_contract_id
        );

        let signed_tx = chain::construct_signed_tx(
            Chain::Near,
            &config.signer_account_id,
            &config.account_secret_key,
            &self.delegation_contract_id,
            "deposit",
            json!({
                "amount": &amount_yocto.to_string(),
                "ed25519_public_key": &ed25519_public_key,
            })
            .to_string()
            .into_bytes(),
            config.gas,
            to_yocto("0.01"),
            &config.rpc_url,
        )
        .await?;

        let config = ChainConfigsInner::test_config();
        let client = Client::new(&Chain::Near, &config)?;
        chain::send_tx(Chain::Near, client, &signed_tx).await?;

        println!("Transaction has been completed");

        Ok(())
    }
}
