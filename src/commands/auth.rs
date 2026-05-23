use std::{io,io::Write};

use crate::cli::AuthCommands;

/// Handler for 'acc auth <subcommand>'
pub async fn handle(subcommand: &AuthCommands) {
    match subcommand {
        AuthCommands::Login => {
            request_user_input("REVAL_SESSION=");
        }
        AuthCommands::Logout => {
            println!("Logging out from AtCoder...");
            // TODO: Add logout logic
        }
        AuthCommands::Status => {
            println!("Checking authentication status...");
            // TODO: Add status check logic
        }
    }
}

fn request_user_input(message: &str) -> String {
    let mut input = String::new();
    print!("{}", message);
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

