use seda_chains::ChainAdapterError;
use seda_config::ConfigError;
use seda_node::NodeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("jsonrpsee client error")]
    JsonRpcClient(#[from] jsonrpsee::core::error::Error),
    #[error(transparent)]
    ChainAdapter(#[from] ChainAdapterError),
    #[error("Config error: {0}")]
    LoadConfig(#[from] ConfigError),
    #[error("Config error: {0}")]
    Config(String),
    #[error(transparent)]
    Node(#[from] NodeError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(debug_assertions)]
    #[error(transparent)]
    CLIDocument(#[from] std::io::Error),
}

impl From<&str> for CliError {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<String> for CliError {
    fn from(value: String) -> Self {
        Self::Config(value)
    }
}

pub type Result<T, E = CliError> = core::result::Result<T, E>;
