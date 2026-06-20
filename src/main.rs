use clap::Parser;

use atcoder_cli::cli::{self, Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Test { task_id } => {
            cli::test::run(&task_id).await;
        }
        Commands::Auth { subcommand } => {
            cli::auth::handle(&subcommand).await;
        }
        Commands::Init(args) => {
            cli::init::init(&args).await;
        }
    }
}
