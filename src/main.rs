use clap::Parser;

use atcoder_cli::commands::{self, Cli, Commands};

#[tokio::main]
async fn main() {
    let args = Cli::parse();

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
