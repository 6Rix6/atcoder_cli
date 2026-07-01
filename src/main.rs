use clap::Parser;

use atcoder_cli::cli::{self, Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Test(args) => {
            cli::test::run(&args).await;
        }
        Commands::Auth(args) => {
            cli::auth::handle(&args).await;
        }
        Commands::Init(args) => {
            cli::init::init(&args).await;
        }
        Commands::Download(args) => {
            cli::dl::download(&args).await;
        }
    }
}
