use thiserror::Error;

#[derive(Error, Debug)]
pub enum TomlError {
    #[error("Invalid Toml Deserialization: {0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("Invalid Toml Serialization: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config io error: {0}")]
    ConfigIoError(#[from] std::io::Error),
    #[error(transparent)]
    InvalidTomlConfig(#[from] TomlError),
    #[error("The field `{0}` must be provided.")]
    MustProvideField(String),
    #[error("Failed to get current directory for logging file path: `{0}.")]
    FailedToGetCurrentDir(String),
    #[error("Unwanted config field `{0}`, due security concerns")]
    UnwantedConfig(String),
}

impl From<String> for ConfigError {
    fn from(value: String) -> Self {
        Self::MustProvideField(value)
    }
}

impl From<&str> for ConfigError {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

pub type Result<T, E = ConfigError> = core::result::Result<T, E>;
