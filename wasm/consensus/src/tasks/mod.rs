use clap::Subcommand;

mod batch;
mod bridge;

#[derive(Debug, Subcommand)]
pub enum Task {
    Batch(batch::Batch),
    Bridge(bridge::Bridge),
}

impl Task {
    pub fn handle(self) {
        match self {
            Self::Batch(bridge) => bridge.handle(),
            Self::Bridge(bridge) => bridge.handle(),
        }
    }
}
