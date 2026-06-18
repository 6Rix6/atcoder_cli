use clap::Parser;

use atcoder_cli::cli::{self, Cli, Commands};

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Test { task_id } => {
            cli::test::run(&task_id).await;
        }
        Commands::Auth { subcommand } => {
            cli::auth::handle(&subcommand).await;
        }
        Commands::Init => {
            cli::init::init();
        }
    }
}
