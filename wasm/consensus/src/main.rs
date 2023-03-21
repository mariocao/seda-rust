use clap::Parser;
use seda_runtime_sdk::{wasm::log, Level};

mod message;
mod tasks;

#[derive(Debug, Parser)]
struct Options {
    #[command(subcommand)]
    task: tasks::Task,
}

fn main() {
    let options = Options::parse();
    log!(Level::Debug, "options: {options:?}");

    options.task.handle();
}
