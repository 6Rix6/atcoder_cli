use dialoguer::{Input, theme::ColorfulTheme};

use crate::{
    atcoder::client::AtCoderClient,
    cli::{
        AuthCommands,
        styles::{self, spinner::Spinner},
    },
    constants::REVEL_SESSION,
    cookie::{self, storage::CookieStorage},
};

/// Handler for 'acc auth <subcommand>'
pub async fn handle(subcommand: &AuthCommands) {
    match subcommand {
        AuthCommands::Login => {
            info_ln!("Open AtCoder in your browser and log in.");
            info_ln!(
                "Open the developer tools by pressing F12 or right-clicking and selecting Inspect, then go to the {} tab > {} > {} > https://atcoder.jp.",
                styles::dim("Application"),
                styles::dim("Storage"),
                styles::dim("Cookie")
            );
            info_ln!(
                "Copy the value of {} and paste it below.",
                styles::dim(REVEL_SESSION)
            );

            let input = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(REVEL_SESSION)
                .report(false)
                .interact()
                .unwrap_or_else(|e| {
                    error_ln!("An error occurred: {}", e);
                    std::process::exit(1);
                });

            let path = cookie::utils::get_cookie_path();
            let storage = CookieStorage::new(&path);
            let result = storage.set(REVEL_SESSION, &input);

            print!("\n");

            match result {
                Ok(_) => {
                    success_ln!("Saved cookie at: {}", path.to_string_lossy().into_owned());
                }
                Err(_) => {
                    error_ln!("Failed to save cookie. Try again.");
                }
            }
        }
        AuthCommands::Logout => {
            let path = cookie::utils::get_cookie_path();

            if !path.exists() {
                warn_ln!("Cookie file does not exist.");
                return;
            }

            match std::fs::remove_file(path) {
                Ok(_) => {
                    success_ln!("Cookie removed.");
                }
                Err(e) => {
                    error_ln!("Failed to remove cookie: {}", e);
                    std::process::exit(1);
                }
            };
        }
        AuthCommands::Status => {
            let path = cookie::utils::get_cookie_path();

            if !path.exists() || !path.is_file() {
                warn_ln!(
                    "Cookie is not set. Run {} to log in to AtCoder.",
                    styles::dim("`acc auth login`")
                );
                return;
            }

            let storage = CookieStorage::new(path);
            let cookie = match storage.get(REVEL_SESSION) {
                Ok(Some(val)) => Some(val),
                Ok(None) => {
                    warn_ln!(
                        "Necessary cookie ({}) does not exist. Run {} to log in to AtCoder.",
                        REVEL_SESSION,
                        styles::dim("`acc auth login`")
                    );
                    return;
                }
                Err(e) => {
                    error_ln!("An error occurred: {}", e);
                    return;
                }
            };

            let spinner = Spinner::new("Testing cookie...");
            let client = AtCoderClient::new(cookie);

            match client
                .fetch_page("https://atcoder.jp/contests/abc001/submit")
                .await
            {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        spinner.finish(
                            &format!("{}", styles::success("200 OK")),
                            "Cookie is valid.",
                        );
                    } else if status.is_redirection() {
                        spinner.finish(
                            &format!("{}", styles::error(&status.to_string())),
                            "Session expired. Please log in again.",
                        );
                    } else {
                        spinner.finish(
                            &format!("{}", styles::error(&status.to_string())),
                            "Received unexpected status code. Try again.",
                        );
                    }
                }
                Err(e) => {
                    let code = e
                        .status()
                        .map_or_else(|| "ERROR".to_string(), |s| s.to_string());
                    spinner.finish(
                        &format!("{}", styles::error(&code)),
                        "A network error occurred:",
                    );
                    eprintln!("{}", e);
                }
            }
        }
    }
}
