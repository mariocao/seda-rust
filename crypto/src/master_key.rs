use std::{fs::read_to_string, path::Path};

use rand::RngCore;
use serde::{Deserialize, Serialize};

use super::{CryptoError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterKey {
    pub seed: [u8; 32],
}

impl MasterKey {
    pub fn random() -> Self {
        let mut seed = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut seed);

        Self { seed }
    }

    pub fn read_from_path<T: AsRef<Path>>(path: T) -> Result<Self> {
        let hex = read_to_string(path)?;
        let seed = hex::decode(hex)?.as_slice().try_into()?;

        Ok(Self { seed })
    }

    pub fn write_to_path<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let hex = hex::encode(self.seed);
        std::fs::write(path, hex)?;

        Ok(())
    }
}

impl From<MasterKey> for String {
    fn from(master_key: MasterKey) -> Self {
        hex::encode(master_key.seed)
    }
}

impl From<[u8; 32]> for MasterKey {
    fn from(seed: [u8; 32]) -> Self {
        Self { seed }
    }
}

impl From<MasterKey> for [u8; 32] {
    fn from(master_key: MasterKey) -> Self {
        master_key.seed
    }
}

impl TryFrom<&String> for MasterKey {
    type Error = CryptoError;

    fn try_from(hex_string: &String) -> std::result::Result<Self, Self::Error> {
        let seed: [u8; 32] = hex::decode(hex_string)?.as_slice().try_into()?;

        Ok(Self { seed })
    }
}
