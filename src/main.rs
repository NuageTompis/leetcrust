use args::{LCArgs, MainCommand};
use clap::Parser;

mod args;
mod config;
mod create;
mod fetch;
mod parse_api;
mod read_write;
mod solutions;
mod reverse_engineer;
mod find_types;

#[tokio::main]
async fn main() {
    let args = LCArgs::parse();
    match args.arg_type {
        MainCommand::Config(config) => {
            config::handle_config_command(config);
        }
        MainCommand::Create(create) => {
            create::handle_create_command(create).await;
        }
        MainCommand::Fetch(fetch) => {
            fetch::handle_fetch_command(fetch).await;
        }
        MainCommand::FindTypes => {
            find_types::handle_find_types_command();
        }
    }
}
