use std::path::{Path, PathBuf};

use clap::Args;
use seda_config::{create_and_load_or_load_config, NodeConfigInner, FULL_CONFIG_PATH};
use seda_crypto::MasterKey;

use crate::Result;

#[derive(Debug, Args)]
pub struct Init {
    #[clap(default_value = NodeConfigInner::SEDA_SECRET_KEY_PATH, short, long)]
    key_path: String,

    #[clap(default_value = FULL_CONFIG_PATH, short, long)]
    /// The path where the config file should be written to
    config_path: String,
}

impl Init {
    pub fn handle(&self) -> Result<()> {
        let master_key: MasterKey = if let Ok(env_secret_key) = std::env::var("SEDA_SECRET_KEY") {
            MasterKey::try_from(&env_secret_key)?
        } else if Path::new(&self.key_path).exists() {
            MasterKey::read_from_path(&self.key_path)?
        } else {
            let key = MasterKey::random();
            key.write_to_path(&self.key_path)?;
            println!("Written SEDA secret key to {}", &self.key_path);

            key
        };

        let bn254_key = master_key.derive_bn254(0)?;
        let ed25519_key = master_key.derive_ed25519(0)?;

        let ed25519_public_key = ed25519_key.public_key.as_bytes().to_vec();
        let bn254_public_key = bn254_key.public_key.to_compressed()?;

        let account_id = hex::encode(&ed25519_public_key);

        // Generates our template config file
        create_and_load_or_load_config(Some(PathBuf::from(self.config_path.to_string())));

        println!("Key information: \n");
        println!("NEAR Account ID: {account_id}");
        println!(
            "ED25519 Public Key (base58 encoded): ed25519:{}",
            bs58::encode(&ed25519_public_key).into_string()
        );
        println!(
            "BN254 Public Key (base58 encoded): bn254:{}",
            bs58::encode(&bn254_public_key).into_string()
        );

        Ok(())
    }
}
