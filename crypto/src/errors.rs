use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Couldn't read mnemonic phrase from file")]
    Io(#[from] std::io::Error),
    #[error("Couldn't derive key")]
    DerivationE(#[from] concat_kdf::Error),
    #[error("Couldn't convert phrase to mnemonic type: {0}")]
    PhraseConversion(String),
}

pub type Result<T, E = CryptoError> = core::result::Result<T, E>;
