use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Key derivation error: {0}")]
    KeyDerivation(#[from] concat_kdf::Error),
    #[error(transparent)]
    Bn254Error(#[from] bn254::Error),
    #[error("Invalid master key length: {0}")]
    InvalidMasterKeyLength(#[from] std::array::TryFromSliceError),
    #[error("Invalid hex: {0}")]
    FromHex(#[from] hex::FromHexError),
}

pub type Result<T, E = CryptoError> = core::result::Result<T, E>;
