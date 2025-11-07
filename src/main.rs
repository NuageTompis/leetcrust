use std::sync::Mutex;

use args::LCArgs;
use clap::Parser;

use crate::logger::Logger;

mod args;
mod clip;
mod config;
mod create;
mod fetch;
mod linked_list;
mod logger;
mod parse_api;
mod read_write;
mod solutions;
mod test_module;
mod tree;

static LOGGER: Mutex<Logger> = Mutex::new(Logger::new());

#[tokio::main]
async fn main() {
    let args = LCArgs::parse();
    use args::MainCommand as MC;
    match args.arg_type {
        MC::Config(config) => {
            config::handle_config_command(config);
        }
        MC::Create(create) => {
            let _res = create::handle_create_command(create).await;
        }
        MC::Fetch(fetch) => {
            fetch::handle_fetch_command(fetch);
        }
        MC::Clip(clip) => {
            let _res = clip::handle_clip_command(clip).await;
        }
    }
}
