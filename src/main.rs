use clap::Parser;

mod cli;
mod commands;
mod constants;
mod languages;
mod utils;

use crate::cli::Commands;

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();

    match &args.command {
        Commands::Test { task_id } => {
            commands::test::run(&task_id).await;
        }
        Commands::Auth { subcommand } => {
            commands::auth::handle(&subcommand).await;
        }
        Commands::Init => {
            commands::init::init();
        }
    }
}
