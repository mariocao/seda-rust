use bn254::ECDSA;
use clap::Args;
use seda_chains::{chain, Client};
use seda_config::{ChainConfigsInner, DelegateConfig};
use seda_crypto::{derive_bn254_key_pair, derive_ed25519_key_pair};
use seda_runtime_sdk::{Chain, ToBytes};
use serde_json::json;

use crate::cli::utils::to_yocto;

#[derive(Debug, Args)]
pub struct Register {
    /// The contract address to register on
    pub delegation_contract_id: String,
    /// The multi address that is associated with the node
    pub multi_addr:             Option<String>,
}

impl Register {
    pub async fn handle(self, config: DelegateConfig) {
        dbg!(&config.validator_secret_key);
        let bn254_key = derive_bn254_key_pair(&config.validator_secret_key, 0).unwrap();
        let ed25519_key = derive_ed25519_key_pair(&config.validator_secret_key, 0).unwrap();
        let signature = ECDSA::sign(ed25519_key.public_key, &bn254_key.private_key).unwrap();

        // https://docs.near.org/concepts/basics/accounts/creating-accounts#local-implicit-account

        // TODO: Make construct_signed_tx only accept bytes and not strings
        let ed25519_public_key = ed25519_key.public_key.as_bytes().to_vec();
        let ed25519_secret_key_bytes: Vec<u8> = ed25519_key.into();

        dbg!(&hex::encode(&ed25519_public_key));

        let signed_tx = chain::construct_signed_tx(
            Chain::Near,
            &hex::encode(&ed25519_public_key),
            &bs58::encode(ed25519_secret_key_bytes).into_string(),
            &self.delegation_contract_id,
            "register_node",
            json!({
                "multi_addr": self.multi_addr.unwrap_or(String::new()),
                "bn254_public_key": &ed25519_public_key,
                "signature": &signature.to_compressed().unwrap(),
            })
            .to_string()
            .into_bytes(),
            1,
            1000,
            &config.rpc_url,
        )
        .await
        .unwrap();

        println!(
            "Registring {} on contract {}..",
            &hex::encode(&ed25519_public_key),
            self.delegation_contract_id
        );

        let config = ChainConfigsInner::test_config();
        let client = Client::new(&Chain::Near, &config).unwrap();
        chain::send_tx(Chain::Near, client, &signed_tx).await.unwrap();

        println!("Transaction has been completed");

        // let amount_yocto = to_yocto(&self.amount);

        // let signed_tx = chain::construct_transfer_tx(
        //     Chain::Near,
        //     &config.signer_account_id,
        //     &config.account_secret_key,
        //     &self.receiver,
        //     amount_yocto,
        //     &config.rpc_url,
        // )
        // .await
        // .unwrap();

        // let config = ChainConfigsInner::test_config();
        // let client = Client::new(&Chain::Near, &config).unwrap();

        // println!("Sending {}N to {}..", self.amount, self.receiver);
        // chain::send_tx(Chain::Near, client, &signed_tx).await.unwrap();
        // println!("Transaction has been completed");
    }
}
