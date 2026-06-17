use clap::{Parser, Subcommand};

#[macro_use]
mod macros;

pub mod auth;
pub mod init;
pub mod test;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Test { task_id: String },
    Auth {
        #[command(subcommand)]
        subcommand: AuthCommands,
    },
    Init
}

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    Login,
    Logout,
    Status,
}


