use clap::Parser;

mod cli;
mod commands;
mod utils;
mod constants;

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();

    match &args.command {
        cli::Commands::Test { task_id } => {
            commands::test::run(&task_id).await;
        }
        cli::Commands::Auth { subcommand } => {
            commands::auth::handle(&subcommand).await;
        }
    }
}
