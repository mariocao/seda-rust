use clap::Parser;

mod message;
mod tasks;
mod types;

#[derive(Debug, Parser)]
struct Options {
    #[command(subcommand)]
    task: tasks::Task,
}

fn main() {
    let options = Options::parse();
    options.task.handle();
}
