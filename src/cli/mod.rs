use std::io::IsTerminal;

use clap::{Parser, Subcommand};

use crate::cli::{auth::AuthArgs, dl::DlArgs, init::InitArgs, setup::SetupArgs, test::TestArgs};
use crate::error::AppError;

#[macro_use]
mod styles;

mod event;
mod utils;

pub mod auth;
pub mod dl;
pub mod init;
pub mod setup;
pub mod template;
pub mod test;

/// The result of a command that completes normally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The command finished successfully (exit code 0).
    Success,
    /// The command ran to completion but the outcome was negative (exit code 2).
    Failed,
}

/// A CLI helper for AtCoder: download problem statements, run sample tests,
/// and manage your login session.
#[derive(Parser, Debug)]
#[command(name = "acc", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run sample test cases for a task and report AC/WA/TLE/RE verdicts.
    #[clap(visible_alias = "t")]
    Test(TestArgs),

    /// Manage the AtCoder login session (REVEL_SESSION cookie).
    #[clap(visible_alias = "a")]
    Auth(AuthArgs),

    /// Create an atcoder.toml config file interactively.
    Init(InitArgs),

    /// Create a contest directory with one subdirectory per problem.
    #[clap(visible_alias = "s")]
    Setup(SetupArgs),

    /// Download a problem statement as Markdown (stdout by default, or a file with -o).
    #[clap(visible_alias = "dl")]
    Download(DlArgs),
}

/// Runs the requested command and reports its outcome.
///
/// Errors bubble up to `main`, which prints them and selects the exit code.
pub fn run(cli: &Cli) -> Result<Outcome, AppError> {
    let _ = event::set_is_json(cli.json);

    match &cli.command {
        Commands::Test(args) => test::run(args),
        Commands::Auth(args) => auth::handle(args),
        Commands::Init(args) => init::init(args),
        Commands::Setup(args) => setup::run(args),
        Commands::Download(args) => dl::download(args),
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

    event::emit(&event::CommonEvent::Error { message: text });
}

/// Returns an error when no interactive terminal is available.
///
/// dialoguer prompts never complete without a TTY (they re-render forever),
/// so commands that prompt must bail out before asking for input.
pub fn ensure_interactive() -> Result<(), AppError> {
    if std::io::stdin().is_terminal() {
        Ok(())
    } else {
        Err(AppError::NotInteractive)
    }
}
