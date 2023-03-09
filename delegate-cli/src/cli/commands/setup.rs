use clap::Args;
use seda_config::DelegateConfig;
use seda_crypto::derive_ed25519_key_pair;

use super::{register::Register, stake::Stake, top_up::TopUp};

#[derive(Debug, Args)]
pub struct Setup {
    /// The contract address to stake on
    pub delegation_contract_id: String,
    /// Amount of tokens to transfer in wholes (1 = 1 NEAR) (defaults to 5 NEAR)
    pub topup_amount:           Option<String>,
    /// The amount of SEDA tokens to stake (1 = 1 SEDA) (defaults to 32 SEDA)
    pub stake_amount:           Option<String>,
    /// The multi address that is associated with the node (defaults to none)
    pub multi_addr:             Option<String>,
}

impl Setup {
    pub async fn handle(self, config: DelegateConfig) {
        let ed25519_key = derive_ed25519_key_pair(&config.validator_secret_key, 0).unwrap();
        let ed25519_public_key = ed25519_key.public_key.as_bytes().to_vec();

        let top_up = TopUp {
            amount:   self.topup_amount.unwrap_or("5".to_string()),
            receiver: hex::encode(&ed25519_public_key),
        };

        top_up.handle(config.clone()).await;

        let register = Register {
            delegation_contract_id: self.delegation_contract_id.clone(),
            multi_addr:             self.multi_addr,
        };

        register.handle(config.clone()).await;

        let stake = Stake {
            amount:                 self.stake_amount.unwrap_or("32".to_string()),
            delegation_contract_id: self.delegation_contract_id,
        };

        stake.handle(config).await;
    }
}
