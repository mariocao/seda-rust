mod get;
pub use get::*;

mod register;
pub use register::*;

mod update;
pub use update::*;

use super::*;

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct NodeInfo {
    // Changed from near_sdk::AccountId, as near_sdk is not compatible on windows machines.
    pub account_id:         String,
    pub multi_addr:         String,
    pub balance:            u128,
    pub bn254_public_key:   Vec<u8>,
    pub ed25519_public_key: Vec<u8>,
}

impl NodeInfo {
    #[cfg(feature = "rand")]
    pub fn random() -> Self {
        use rand::{distributions::Alphanumeric, thread_rng, Rng};
        let rng = &mut thread_rng();
        let account_id_stem = rng
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect::<String>();
        let master_key = seda_crypto::MasterKey::random();
        Self {
            account_id:         format!("{account_id_stem}.testnet"),
            multi_addr:         format!(
                "{}.{}.{}.{}:{}",
                rng.gen_range(1..255),
                rng.gen_range(1..255),
                rng.gen_range(1..255),
                rng.gen_range(1..255),
                rng.gen_range(1..65535)
            ),
            balance:            rng.gen_range(0..u128::MAX),
            bn254_public_key:   master_key.derive_bn254(0).unwrap().public_key.to_compressed().unwrap(),
            ed25519_public_key: master_key.derive_ed25519(0).unwrap().public_key.as_bytes().to_vec(),
        }
    }
}
