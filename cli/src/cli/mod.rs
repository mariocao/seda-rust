use std::path::PathBuf;

use clap::{arg, command, Parser, Subcommand};
use seda_config::{AppConfig, PartialChainConfigs, PartialNodeConfig};
use seda_runtime_sdk::Chain;

use crate::Result;

// mod cli_commands;
// use cli_commands::*;

// mod near_backend;

#[derive(Parser)]
#[command(name = "seda")]
#[command(author = "https://github.com/SedaProtocol")]
#[command(version = "0.1.0")]
#[command(about = "For interacting with the SEDA protocol.", long_about = None)]
pub struct CliOptions {
    #[arg(short, long)]
    chain:             Chain,
    #[arg(long)]
    pub log_file_path: Option<PathBuf>,
    #[command(subcommand)]
    command:           Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run {
        #[command(flatten)]
        node_config:   PartialNodeConfig,
        #[command(flatten)]
        chains_config: PartialChainConfigs,
    },
    Cli {
        args: Vec<String>,
    },
    RegisterNode {
        #[arg(short, long)]
        socket_address:      String,
        #[arg(long)]
        seda_server_url:     Option<String>,
        #[arg(long)]
        signer_account_id:   Option<String>,
        #[arg(short = 'k', long)]
        secret_key:          Option<String>,
        #[arg(short, long)]
        contract_account_id: Option<String>,
    },
    GetNodes {
        #[arg(short, long)]
        limit:               u64,
        #[arg(short, long, default_value = "0")]
        offset:              u64,
        #[arg(short, long)]
        contract_account_id: Option<String>,
    },
    GetNodeSocketAddress {
        #[arg(short, long)]
        node_id:             u64,
        #[arg(short, long)]
        contract_account_id: Option<String>,
    },
    RemoveNode {
        #[arg(short, long)]
        node_id:             u64,
        #[arg(long)]
        seda_server_url:     Option<String>,
        #[arg(long)]
        signer_account_id:   Option<String>,
        #[arg(short = 'k', long)]
        secret_key:          Option<String>,
        #[arg(short, long)]
        contract_account_id: Option<String>,
    },
    SetNodeSocketAddress {
        #[arg(short, long)]
        node_id:             u64,
        #[arg(short, long)]
        socket_address:      String,
        #[arg(long)]
        seda_server_url:     Option<String>,
        #[arg(long)]
        signer_account_id:   Option<String>,
        #[arg(short = 'k', long)]
        secret_key:          Option<String>,
        #[arg(short, long)]
        contract_account_id: Option<String>,
    },
    GetNodeOwner {
        #[arg(short, long)]
        node_id:             u64,
        #[arg(short, long)]
        contract_account_id: Option<String>,
    },
    SignTxn {
        #[arg(short, long)]
        signer_account_id:   Option<String>,
        #[arg(short = 'k', long)]
        secret_key:          Option<String>,
        #[arg(short, long)]
        contract_account_id: Option<String>,
        #[arg(short, long)]
        method_name:         String,
        #[arg(short, long)]
        args:                String,
        #[arg(short, long)]
        gas:                 u64,
        #[arg(short, long)]
        deposit:             u128,
    },
}

impl CliOptions {
    // This is temporary until we move the execution of these to
    // the runtime.
    // #[tokio::main]
    // async fn rest_of_options<T: CliCommands>(command: Command) -> Result<()> {
    //     match command {
    //         // cargo run cli call mc.mennat0.testnet register_node
    // "{\"socket_address\":\"127.0.0.1:8080\"}"         //
    // "870000000000000000000"         Command::RegisterNode {
    //             socket_address,
    //             seda_server_url,
    //             signer_account_id,
    //             secret_key,
    //             contract_account_id,
    //         } => {
    //             {
    //                 let mut config = CONFIG.blocking_write();
    //                 if let Some(seda_server_url) = seda_server_url {
    //                     config.seda_server_url = seda_server_url;
    //                 }

    //                 if let Some(signer_account_id) = signer_account_id {
    //                     config.node.signer_account_id = signer_account_id;
    //                 }
    //                 if let Some(secret_key) = secret_key {
    //                     config.node.secret_key = secret_key;
    //                 }
    //                 if let Some(contract_account_id) = contract_account_id {
    //                     config.node.contract_account_id = contract_account_id;
    //                 }
    //             }
    //             T::register_node(socket_address).await?
    //         }
    //         // cargo run --bin seda get-nodes --limit 2
    //         Command::GetNodes {
    //             limit,
    //             offset,
    //             contract_account_id,
    //         } => {
    //             {
    //                 let mut config = CONFIG.blocking_write();
    //                 if let Some(contract_account_id) = contract_account_id {
    //                     config.node.contract_account_id = contract_account_id;
    //                 }
    //             }
    //             T::get_nodes(limit, offset).await?
    //         }
    //         // cargo run --bin seda get-node-socket-address --node-id 9
    //         Command::GetNodeSocketAddress {
    //             node_id,
    //             contract_account_id,
    //         } => {
    //             {
    //                 let mut config = CONFIG.blocking_write();
    //                 if let Some(contract_account_id) = contract_account_id {
    //                     config.node.contract_account_id = contract_account_id;
    //                 }
    //             }
    //             T::get_node_socket_address(node_id).await?
    //         }
    //         // cargo run --bin seda remove-node --node-id 9
    //         Command::RemoveNode {
    //             node_id,
    //             seda_server_url,
    //             signer_account_id,
    //             secret_key,
    //             contract_account_id,
    //         } => {
    //             {
    //                 let mut config = CONFIG.blocking_write();
    //                 if let Some(seda_server_url) = seda_server_url {
    //                     config.seda_server_url = seda_server_url;
    //                 }

    //                 if let Some(signer_account_id) = signer_account_id {
    //                     config.node.signer_account_id = signer_account_id;
    //                 }
    //                 if let Some(secret_key) = secret_key {
    //                     config.node.secret_key = secret_key;
    //                 }
    //                 if let Some(contract_account_id) = contract_account_id {
    //                     config.node.contract_account_id = contract_account_id;
    //                 }
    //             }
    //             T::remove_node(node_id).await?
    //         }
    //         // cargo run --bin seda set-node-socket-address --node-id 9
    //         Command::SetNodeSocketAddress {
    //             node_id,
    //             socket_address,
    //             seda_server_url,
    //             signer_account_id,
    //             secret_key,
    //             contract_account_id,
    //         } => {
    //             {
    //                 let mut config = CONFIG.blocking_write();
    //                 if let Some(seda_server_url) = seda_server_url {
    //                     config.seda_server_url = seda_server_url;
    //                 }

    //                 if let Some(signer_account_id) = signer_account_id {
    //                     config.node.signer_account_id = signer_account_id;
    //                 }
    //                 if let Some(secret_key) = secret_key {
    //                     config.node.secret_key = secret_key;
    //                 }
    //                 if let Some(contract_account_id) = contract_account_id {
    //                     config.node.contract_account_id = contract_account_id;
    //                 }
    //             }
    //             T::set_node_socket_address(node_id, socket_address).await?
    //         }
    //         // cargo run --bin seda get-node-owner --node-id 9
    //         Command::GetNodeOwner {
    //             node_id,
    //             contract_account_id,
    //         } => {
    //             {
    //                 let mut config = CONFIG.blocking_write();
    //                 if let Some(contract_account_id) = contract_account_id {
    //                     config.node.contract_account_id = contract_account_id;
    //                 }
    //             }
    //             T::get_node_owner(node_id).await?
    //         }
    //         Command::Cli { args } => T::call_cli(&args).await?,

    //         // The commands `run` and `generate-config` are already handled.
    //         _ => unreachable!(),
    //     }

    //     Ok(())
    // }

    pub fn handle(self, config: AppConfig) -> Result<()> {
        if let Command::Run {
            node_config,
            chains_config,
        } = self.command
        {
            let node_config = config.node.to_config(node_config)?;
            let chains_config = config.chains.to_config(chains_config)?;
            seda_node::run(node_config, chains_config);

            return Ok(());
        }

        unimplemented!()
        // Self::rest_of_options::<near_backend::NearCliBackend>(options.
        // command)
    }
}
