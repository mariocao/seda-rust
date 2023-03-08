use std::fs::read_to_string;

use bn254::{PrivateKey as Bn254PrivateKey, PublicKey as Bn254PublicKey};
use concat_kdf::derive_key;
use ed25519_dalek::{PublicKey as Ed25519PublicKey, SecretKey as Ed25519PrivateKey, SECRET_KEY_LENGTH};
mod errors;
use crate::errors::CryptoError;
#[derive(PartialEq)]
pub enum KeyType {
    Ed25519,
    Bn254,
}

#[allow(dead_code)]
pub struct Ed25519KeyPair {
    pub public_key:  Ed25519PublicKey,
    pub private_key: Ed25519PrivateKey,
}

#[allow(dead_code)]
pub struct Bn254KeyPair {
    pub public_key:  Bn254PublicKey,
    pub private_key: Bn254PrivateKey,
}

impl From<Ed25519KeyPair> for Vec<u8> {
    fn from(val: Ed25519KeyPair) -> Self {
        let mut result = vec![];

        result.extend_from_slice(val.private_key.as_bytes());
        result.extend_from_slice(val.public_key.as_bytes());

        result
    }
}

pub fn derive_bn254_key_pair_from_fs(sk_path: &str, index: usize) -> Result<Bn254KeyPair, CryptoError> {
    let seed = read_to_string(sk_path)?;

    derive_bn254_key_pair(&seed, index)
}

pub fn derive_bn254_key_pair(seed: &str, index: usize) -> Result<Bn254KeyPair, CryptoError> {
    let master_sk = derive_key::<sha2::Sha256>(seed.as_bytes(), b"bn254", SECRET_KEY_LENGTH)?;
    let sk = derive_key::<sha2::Sha256>(master_sk.as_slice(), &index.to_ne_bytes(), SECRET_KEY_LENGTH)?;
    let private_key = Bn254PrivateKey::try_from(sk.as_slice()).unwrap();
    let public_key = Bn254PublicKey::from_private_key(&private_key);

    Ok(Bn254KeyPair {
        public_key,
        private_key,
    })
}

pub fn derive_ed25519_key_pair_from_fs(sk_path: &str, index: usize) -> Result<Ed25519KeyPair, CryptoError> {
    let seed = read_to_string(sk_path)?;

    derive_ed25519_key_pair(&seed, index)
}

pub fn derive_ed25519_key_pair(seed: &str, index: usize) -> Result<Ed25519KeyPair, CryptoError> {
    let master_sk = derive_key::<sha2::Sha256>(seed.as_bytes(), b"ed25519", SECRET_KEY_LENGTH)?;
    let sk = derive_key::<sha2::Sha256>(master_sk.as_slice(), &index.to_ne_bytes(), SECRET_KEY_LENGTH)?;
    let private_key = Ed25519PrivateKey::from_bytes(sk.as_slice()).unwrap();
    let public_key: Ed25519PublicKey = (&private_key).into();

    Ok(Ed25519KeyPair {
        public_key,
        private_key,
    })
}

#[cfg(test)]
#[path = ""]
pub mod test {
    mod crypto_test;
}
