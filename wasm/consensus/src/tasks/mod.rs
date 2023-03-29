use clap::Subcommand;

mod batch;
mod bridge;
mod p2p;

#[derive(Debug, Subcommand)]
pub enum Task {
    Batch(batch::Batch),
    Bridge(bridge::Bridge),
    P2P(p2p::P2P),
}

impl Task {
    pub fn handle(self) {
        match self {
            Self::Batch(batch) => batch.handle(),
            Self::Bridge(bridge) => bridge.handle(),
            Self::P2P(p2p) => p2p.handle(),
        }
    }
}
