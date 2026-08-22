use std::process::ExitCode;

use clap::Parser;

use atcoder_cli::cli::{self, Cli, Outcome};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let outcome = match cli::run(&cli) {
        Ok(outcome) => outcome,
        Err(err) => {
            cli::report_error(&err);
            return ExitCode::FAILURE;
        }
    };

    match outcome {
        Outcome::Success => ExitCode::SUCCESS,
        Outcome::Failed => ExitCode::from(2),
    }
}
