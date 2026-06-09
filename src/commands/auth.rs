use std::{fs, io, io::Write};

use crate::cli::AuthCommands;
use crate::utils::cookie_storage;
use crate::utils::scraper;

const SESSION_KEY: &str = "REVAL_SESSION";

/// Handler for 'acc auth <subcommand>'
pub async fn handle(subcommand: &AuthCommands) {
    match subcommand {
        AuthCommands::Login => {
            let cookie = request_user_input("REVAL_SESSION=");

            let path = cookie_storage::get_cookie_path();
            let storage = cookie_storage::CookieStorage::new(&path);
            let result = storage.set(SESSION_KEY, &cookie);

            match result {
                Ok(_) => {
                    println!("Saved cookie at: {}", path.to_string_lossy().into_owned())
                }
                Err(_) => {
                    println!("Failed to save cookie. Try again.")
                }
            }
        }
        AuthCommands::Logout => {
            let path = cookie_storage::get_cookie_path();

            if !path.exists() {
                println!("Cookie does not set.");
                return;
            }

            match fs::remove_file(path) {
                Ok(_) => {
                    println!("Cookie removed.");
                }
                Err(e) => {
                    eprintln!("Failed to remove cookie: {}", e);
                }
            };
        }
        AuthCommands::Status => {
            let path = cookie_storage::get_cookie_path();

            if !path.exists() || !path.is_file() {
                println!("Cookie does not set. You can set one with `acc auth login`.");
                return;
            }

            let storage = cookie_storage::CookieStorage::new(path);
            let session_key = match storage.get(SESSION_KEY) {
                Ok(Some(val)) => val,
                Ok(None) => {
                    println!("");
                    return;
                }
                Err(e) => {
                    eprintln!("An error occurred: {}", e);
                    return;
                }
            };

            match scraper::fetch_page(
                "https://atcoder.jp/contests/abc001/submit",
                Some(&session_key),
            )
            .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("OK");
                        return;
                    }

                    if response.status().is_redirection() {
                        println!("NG");
                        return;
                    }

                    println!("Received other status code: {}", response.status())
                }
                Err(e) => {
                    eprintln!("Network error occurred: {}", e);
                }
            }
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
