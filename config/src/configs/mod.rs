mod logger;
pub use logger::*;

mod macros;
pub use macros::*;

mod main_chain;
pub use main_chain::*;

mod node;
pub use node::*;

mod p2p;
pub use p2p::*;

mod delegate;
pub use delegate::*;

pub trait Config: std::fmt::Debug + Default + serde::Serialize + serde::de::DeserializeOwned {
    fn template() -> Self;
    fn overwrite_from_env(&mut self) {}
}
