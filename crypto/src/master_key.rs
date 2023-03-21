use std::{fs::read, path::Path};

use rand::RngCore;

use crate::{errors::Result, CryptoError};

struct MasterKey {
    pub seed: [u8; 32],
}

impl MasterKey {
    pub fn random() -> Self {
        let mut seed = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut seed);

        Self { seed }
    }

    pub fn from_fs<T: AsRef<Path>>(path: T) -> Result<Self> {
        // TODO: remove unwrap
        // let seed: [u8;32] = read(path)?.try_into().map_err(|e|
        // CryptoError::MasterKey(e))?;
        let seed: [u8; 32] = read(path)?.try_into().unwrap();

        Ok(Self { seed })
    }

    pub fn to_fs<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let hex = hex::encode(self.seed);
        std::fs::write(path, hex)?;

        Ok(())
    }
}

impl From<[u8; 32]> for MasterKey {
    fn from(seed: [u8; 32]) -> Self {
        Self { seed }
    }
}

impl TryFrom<&str> for MasterKey {
    type Error = CryptoError;

    fn try_from(hex_string: &str) -> std::result::Result<Self, Self::Error> {
        // TODO: remove unwraps
        let seed: [u8; 32] = hex::decode(hex_string).unwrap().try_into().unwrap();

        Ok(Self { seed })
    }
}
