use clap::{Parser, Subcommand};

use crate::cli::{auth::AuthArgs, dl::DlArgs, init::InitArgs, test::TestArgs};

#[macro_use]
mod styles;

mod utils;

pub mod auth;
pub mod dl;
pub mod init;
pub mod test;

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
