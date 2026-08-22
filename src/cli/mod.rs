use clap::{Parser, Subcommand};

use crate::cli::{auth::AuthArgs, dl::DlArgs, init::InitArgs, test::TestArgs};
use crate::error::AppError;

#[macro_use]
mod styles;

mod utils;

pub mod auth;
pub mod dl;
pub mod init;
pub mod test;

/// The result of a command that completes normally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The command finished successfully (exit code 0).
    Success,
    /// The command ran to completion but the outcome was negative (exit code 2).
    Failed,
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(visible_alias = "t")]
    Test(TestArgs),

    #[clap(visible_alias = "a")]
    Auth(AuthArgs),

    Init(InitArgs),

    #[clap(visible_alias = "dl")]
    Download(DlArgs),
}

/// Runs the requested command and reports its outcome.
///
/// Errors bubble up to `main`, which prints them and selects the exit code.
pub async fn run(cli: &Cli) -> Result<Outcome, AppError> {
    match &cli.command {
        Commands::Test(args) => test::run(args).await,
        Commands::Auth(args) => auth::handle(args).await,
        Commands::Init(args) => init::init(args).await,
        Commands::Download(args) => dl::download(args).await,
    }
}

/// Prints an error and its full source chain to stderr.
pub fn report_error(err: &AppError) {
    use std::error::Error as _;

    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(s) = source {
        text.push_str(": ");
        text.push_str(&s.to_string());
        source = s.source();
    }

    eprintln!("[{}] {}", styles::error(styles::ERROR_PREFIX), text);
}
