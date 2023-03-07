use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{env_overwrite, ConfigError};
#[cfg(feature = "delegate-cli")]
use crate::{merge_config_cli, Config, Result};

#[cfg(feature = "delegate-cli")]
#[derive(clap::Args)]
/// The configuration for the seda engine.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PartialDelegateConfig {
    /// An option to override the validator secret key config value.
    #[arg(long)]
    pub validator_secret_key: Option<String>,
    /// An option to override the account secret key config value. (Used to sign
    /// top-ups and stakings)
    #[arg(long)]
    pub account_secret_key:   Option<String>,
    /// An option to override the signer account ID config value.
    #[arg(long)]
    pub signer_account_id:    Option<String>,
    /// An option to override the delegate contract id
    #[arg(long)]
    pub delegate_contract_id: Option<String>,
    /// An option to override the RPC URL
    #[arg(long)]
    pub rpc_url:              Option<String>,
}

#[cfg(feature = "delegate-cli")]
impl PartialDelegateConfig {
    pub fn to_config(self, cli_options: Self) -> Result<DelegateConfig> {
        let validator_secret_key = merge_config_cli!(self, cli_options, validator_secret_key, Ok(String::new()))?;
        let account_secret_key = merge_config_cli!(
            self,
            cli_options,
            account_secret_key,
            Err(ConfigError::from("node.secret_key"))
        )?;
        let signer_account_id = merge_config_cli!(
            self,
            cli_options,
            signer_account_id,
            Err(ConfigError::from("node.signer_account_id"))
        )?;

        let delegate_contract_id = merge_config_cli!(self, cli_options, delegate_contract_id, Ok(String::new()))?;
        let rpc_url = merge_config_cli!(self, cli_options, rpc_url, Ok(DelegateConfigInner::RPC_URL.to_string()))?;

        Ok(Arc::new(DelegateConfigInner {
            validator_secret_key,
            account_secret_key,
            signer_account_id,
            delegate_contract_id,
            rpc_url,
        }))
    }
}

#[cfg(feature = "delegate-cli")]
impl Config for PartialDelegateConfig {
    fn template() -> Self {
        Self {
            validator_secret_key: None,
            delegate_contract_id: None,
            account_secret_key:   None,
            signer_account_id:    None,
            rpc_url:              None,
        }
    }

    fn overwrite_from_env(&mut self) {
        env_overwrite!(self.validator_secret_key, "VALIDATOR_SECRET_KEY");
        env_overwrite!(self.account_secret_key, "ACCOUNT_SECRET_KEY");
        env_overwrite!(self.signer_account_id, "SIGNER_ACCOUNT_ID");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateConfigInner {
    pub account_secret_key:   String,
    pub validator_secret_key: String,
    pub signer_account_id:    String,
    pub delegate_contract_id: String,
    pub rpc_url:              String,
}

impl DelegateConfigInner {
    // TODO cfg this
    pub fn test_config() -> DelegateConfig {
        Arc::new(Self {
            delegate_contract_id: String::new(),
            rpc_url:              String::new(),
            validator_secret_key: String::new(),
            account_secret_key:   String::new(),
            signer_account_id:    String::new(),
        })
    }

    pub fn from_json_str(s: &str) -> DelegateConfig {
        let this = serde_json::from_str(s).unwrap();
        Arc::new(this)
    }
}

impl DelegateConfigInner {
    pub const RPC_URL: &str = "https://rpc.testnet.near.org";
}

pub type DelegateConfig = Arc<DelegateConfigInner>;