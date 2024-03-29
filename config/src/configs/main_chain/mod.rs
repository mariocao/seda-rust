//! Defines a ChainConfig type based on features when compiling.

mod near;
use std::sync::Arc;

pub use near::*;

mod another_config;
pub use another_config::*;
#[cfg(feature = "cli")]
use {
    crate::{Config, Result},
    serde::{Deserialize, Serialize},
};

#[cfg(feature = "cli")]
#[derive(clap::Args, Debug, Default, Serialize, Deserialize)]
pub struct PartialChainConfigs {
    #[arg(skip)]
    pub another: PartialAnotherConfig,
    #[command(flatten)]
    pub near:    PartialNearConfig,
}

#[cfg(feature = "cli")]
impl PartialChainConfigs {
    pub fn to_config(self, cli_options: PartialChainConfigs) -> Result<ChainConfigs> {
        Ok(Arc::new(ChainConfigsInner {
            another: self.another.to_config(),
            near:    self.near.to_config(cli_options.near)?,
        }))
    }
}

#[cfg(feature = "cli")]
impl Config for PartialChainConfigs {
    fn template() -> Self {
        PartialChainConfigs {
            another: PartialAnotherConfig::template(),
            near:    PartialNearConfig::template(),
        }
    }

    fn overwrite_from_env(&mut self) {
        self.another.overwrite_from_env();
        self.near.overwrite_from_env();
    }
}

#[derive(Debug, Clone)]
pub struct ChainConfigsInner {
    pub another: AnotherConfig,
    pub near:    NearConfig,
}

impl ChainConfigsInner {
    // TODO cfg this
    pub fn test_config() -> Arc<Self> {
        Arc::new(Self {
            another: AnotherConfig::test_config(),
            near:    NearConfig::test_config(),
        })
    }
}

pub type ChainConfigs = Arc<ChainConfigsInner>;
