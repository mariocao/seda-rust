use thiserror::Error;

#[derive(Debug, Error)]
pub enum SDKError {
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("{0:?}")]
    StringBytesConversion(#[from] std::str::Utf8Error),

    #[error(transparent)]
    NumBytesConversion(#[from] std::array::TryFromSliceError),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error("Expected a valid url scheme but got `{0}`")]
    InvalidUrlScheme(String),
}

pub type Result<T, E = SDKError> = core::result::Result<T, E>;
