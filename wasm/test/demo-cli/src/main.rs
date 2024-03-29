use clap::{Parser, Subcommand};
use seda_runtime_sdk::{
    wasm::{call_self, chain_call, chain_view, db_set, http_fetch, Promise},
    Chain,
    FromBytes,
    PromiseStatus,
};

#[derive(Parser)]
#[command(name = "seda")]
#[command(author = "https://github.com/SedaProtocol")]
#[command(version = "0.1.0")]
#[command(about = "For interacting with the SEDA protocol.", long_about = None)]
struct Options {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Hello,
    HttpFetch {
        url: String,
    },
    View {
        chain:       Chain,
        contract_id: String,
        method_name: String,
        args:        String,
    },
    Call {
        chain:       Chain,
        contract_id: String,
        method_name: String,
        args:        String,
        deposit:     u128,
    },
}

fn main() {
    let options = Options::parse();

    if let Some(command) = options.command {
        match command {
            // cargo run cli http-fetch "https://www.breakingbadapi.com/api/characters/1"
            Commands::HttpFetch { url } => {
                http_fetch(&url).start().then(call_self("http_fetch_result", vec![]));
            }
            Commands::Hello => {
                println!("Hello World from inside wasm");
            }
            //cargo run cli view mc.mennat0.testnet get_node_owner "{\"node_id\":\"12\"}"
            Commands::View {
                chain,
                contract_id,
                method_name,
                args,
            } => {
                chain_view(chain, contract_id, method_name, args.into_bytes())
                    .start()
                    .then(call_self("chain_view_test_success", vec![]));
            }
            // cargo run cli call mc.mennat0.testnet register_node "{\"socket_address\":\"127.0.0.1:8080\"}"
            // "870000000000000000000"
            Commands::Call {
                chain,
                contract_id,
                method_name,
                args,
                deposit,
            } => {
                chain_call(chain, contract_id, method_name, args.into_bytes(), deposit)
                    .start()
                    .then(call_self("chain_call_test_success", vec![]));
            }
        }
    }
}

#[no_mangle]
fn http_fetch_result() {
    let result = Promise::result(0);

    let value_to_store: String = match result {
        PromiseStatus::Fulfilled(Some(vec)) => String::from_bytes_vec(vec).unwrap(),
        _ => "Promise failed..".to_string(),
    };

    println!("Value: {value_to_store}");
}

#[no_mangle]
fn chain_view_test_success() {
    let result = Promise::result(0);
    let value_to_store: String = match result {
        PromiseStatus::Fulfilled(Some(vec)) => String::from_bytes_vec(vec).unwrap(),
        _ => "Promise failed..".to_string(),
    };
    println!("Value: {value_to_store}");

    db_set("chain_view_result", &value_to_store).start();
}

#[no_mangle]
fn chain_call_test_success() {
    let result = Promise::result(0);
    let value_to_store: String = match result {
        PromiseStatus::Fulfilled(Some(vec)) => String::from_bytes_vec(vec).unwrap(),
        _ => "Promise failed..".to_string(),
    };
    println!("Value: {value_to_store}");
    db_set("chain_call_result", &value_to_store).start();
}
