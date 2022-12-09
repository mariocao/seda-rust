//! Defines a MainChain type based on features when compiling.

mod errors;
pub use errors::*;

mod mainchain_adapter_trait;
pub use mainchain_adapter_trait::*;

#[cfg(feature = "near")]
mod near_mainchain;

#[cfg(feature = "near")]
pub type MainChain = near_mainchain::NearMainChain;