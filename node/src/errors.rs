use seda_chain_adapters::MainChainAdapterError;
use seda_p2p_adapters::P2PAdapterError;
use thiserror::Error;
use actix::MailboxError;
#[derive(Error, Debug)]
pub enum NodeError {
    #[error(transparent)]
    RPCError(#[from] jsonrpsee::core::Error),
    #[error(transparent)]
    MainChainError(#[from] MainChainAdapterError),
    #[error(transparent)]
    P2PError(#[from] P2PAdapterError),
    MailboxError(#[from] MailboxError),
}

pub type Result<T, E = NodeError> = core::result::Result<T, E>;
