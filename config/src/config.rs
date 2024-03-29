use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    env_overwrite,
    errors::{Result, TomlError},
    Config,
};
#[cfg(feature = "cli")]
use crate::{PartialChainConfigs, PartialLoggerConfig, PartialNodeConfig, PartialP2PConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialAppConfig {
    pub seda_server_address: String,
    pub seda_server_port:    u64,
    #[cfg(feature = "cli")]
    pub chains:              PartialChainConfigs,
    #[cfg(feature = "cli")]
    pub node:                PartialNodeConfig,
    #[cfg(feature = "cli")]
    pub logging:             PartialLoggerConfig,
    #[cfg(feature = "cli")]
    pub p2p:                 PartialP2PConfig,
}

impl Default for PartialAppConfig {
    fn default() -> Self {
        let mut this = Self {
            seda_server_address:             "127.0.0.1".to_string(),
            seda_server_port:                12345,
            #[cfg(feature = "cli")]
            chains:                          PartialChainConfigs::default(),
            #[cfg(feature = "cli")]
            node:                            PartialNodeConfig::default(),
            #[cfg(feature = "cli")]
            logging:                         PartialLoggerConfig::default(),
            #[cfg(feature = "cli")]
            p2p:                             PartialP2PConfig::default(),
        };
        this.overwrite_from_env();
        this
    }
}

impl Config for PartialAppConfig {
    fn template() -> Self {
        Self {
            seda_server_address:             "127.0.0.1".to_string(),
            seda_server_port:                12345,
            #[cfg(feature = "cli")]
            chains:                          PartialChainConfigs::template(),
            #[cfg(feature = "cli")]
            node:                            PartialNodeConfig::template(),
            #[cfg(feature = "cli")]
            logging:                         PartialLoggerConfig::template(),
            #[cfg(feature = "cli")]
            p2p:                             PartialP2PConfig::template(),
        }
    }

    fn overwrite_from_env(&mut self) {
        env_overwrite!(self.seda_server_address, "SEDA_SERVER_ADDRESS");
        env_overwrite!(self.seda_server_port, "SEDA_SERVER_PORT", |p: String| p
            .parse()
            .expect("Invalid port number specified."));
        #[cfg(feature = "cli")]
        self.chains.overwrite_from_env();
        #[cfg(feature = "cli")]
        self.node.overwrite_from_env();
        #[cfg(feature = "cli")]
        self.logging.overwrite_from_env();
    }
}

impl PartialAppConfig {
    /// For reading from a toml file.
    pub fn from_read<R: std::io::Read>(buf: &mut R) -> Result<Self> {
        let mut content = String::new();
        buf.read_to_string(&mut content)?;
        let mut config: Self = toml::from_str(&content).map_err(TomlError::Deserialize)?;
        config.overwrite_from_env();
        Ok(config)
    }

    /// For reading from a toml file from a path.
    pub fn read_from_path(path: PathBuf) -> Result<Self> {
        let mut file = std::fs::OpenOptions::new().read(true).open(path)?;
        Self::from_read(&mut file)
    }

    /// For writing a default configuration file.
    pub fn write_template<W: std::io::Write>(buf: &mut W) -> Result<()> {
        let template = Self::template();
        let content = toml::to_string_pretty(&template).map_err(TomlError::Serialize)?;
        buf.write_all(content.as_bytes())?;
        Ok(())
    }

    /// For creating a default config to a given path.
    pub fn create_template_from_path(path: &PathBuf) -> Result<()> {
        if let Some(prefix) = path.parent() {
            if !prefix.exists() {
                std::fs::create_dir_all(prefix)?;
            }
        }
        let mut file = std::fs::OpenOptions::new().create(true).write(true).open(path)?;
        Self::write_template(&mut file)
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub seda_server_url: String,
    #[cfg(feature = "cli")]
    pub chains:          PartialChainConfigs,
    #[cfg(feature = "cli")]
    pub node:            PartialNodeConfig,
    #[cfg(feature = "cli")]
    pub p2p:             PartialP2PConfig,
}

impl AsRef<AppConfig> for AppConfig {
    fn as_ref(&self) -> &Self {
        self
    }
}
#[cfg(feature = "cli")]
impl From<PartialAppConfig> for (AppConfig, PartialLoggerConfig) {
    fn from(value: PartialAppConfig) -> Self {
        (
            AppConfig {
                seda_server_url:                format!("{}:{}", value.seda_server_address, value.seda_server_port),
                #[cfg(feature = "cli")]
                chains:                         value.chains,
                #[cfg(feature = "cli")]
                node:                           value.node,
                #[cfg(feature = "cli")]
                p2p:                            value.p2p,
            },
            #[cfg(feature = "cli")]
            value.logging,
        )
    }
}
