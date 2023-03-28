use bn254::Error as Bn254Error;
use seda_chains::ChainAdapterError;
use seda_crypto::CryptoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DelegateCliError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Could not convert key: {0}")]
    Crypto(#[from] CryptoError),

    #[error("BN254 operation failed: {0}")]
    Bn254(#[from] Bn254Error),

    #[error("Using the chain interface threw an error: {0}")]
    Chain(#[from] ChainAdapterError),
}

pub type Result<T, E = DelegateCliError> = core::result::Result<T, E>;
