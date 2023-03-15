use super::*;

// TODO how to handle this better? Contract wants a tuple style enum, but CLI
// needs a struct style enum. Talk with @jameson
/// Update node commands
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "cli", derive(clap::Subcommand))]
pub enum UpdateNode {
    SetSocketAddress { new_multi_addr: String },
}

impl ToString for UpdateNode {
    fn to_string(&self) -> String {
        let json = json!(self);
        json.to_string()
    }
}
