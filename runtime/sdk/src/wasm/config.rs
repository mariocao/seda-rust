use lazy_static::lazy_static;
use seda_config::{NodeConfig, NodeConfigInner};

fn config() -> NodeConfig {
    for (key, value) in std::env::vars() {
        dbg!(key, value);
    }
    dbg!(std::env::var("WASM_NODE_CONFIG"));
    let config_str = std::env::var("WASM_NODE_CONFIG").expect("ENV DNE");
    NodeConfigInner::from_json_str(&config_str)
}

// Lazy static so its only converting from bytes once per wasm bin
lazy_static! {
    pub static ref CONFIG: NodeConfig = config();
}
