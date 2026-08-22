use clap::{Args, Subcommand};
use dialoguer::Input;

use crate::{
    atcoder::client::AtCoderClient,
    cli::{
        self, Outcome,
        styles::{self, ERROR_PREFIX, dialog_theme::DialogTheme, spinner::Spinner},
    },
    constants::REVEL_SESSION,
    cookie::{
        error::CookieError,
        storage::CookieStorage,
        utils::{get_cookie_path, is_valid_session_value},
    },
    error::AppError,
};

/// Manage the AtCoder login session.
#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    subcommand: AuthCommands,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    /// Save the REVEL_SESSION cookie by pasting it from your browser.
    Login,
    /// Delete the saved cookie file.
    Logout,
    /// Check whether the saved cookie is still valid.
    Status,
}

/// Handler for 'acc auth <subcommand>'
pub fn handle(args: &AuthArgs) -> Result<Outcome, AppError> {
    match &args.subcommand {
        AuthCommands::Login => login(),
        AuthCommands::Logout => logout(),
        AuthCommands::Status => status(),
    }
}

fn login() -> Result<Outcome, AppError> {
    cli::ensure_interactive()?;

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

    let input = Input::<String>::with_theme(&DialogTheme::default())
        .with_prompt(REVEL_SESSION)
        .report(false)
        .validate_with(|input: &String| {
            if is_valid_session_value(input.trim()) {
                Ok(())
            } else {
                Err(
                    "Value must be non-empty printable ASCII (no spaces or symbols \
                    that are invalid in a cookie value).",
                )
            }
        })
        .interact()?;

    let input = input.trim();

    let path = get_cookie_path()?;
    let storage = CookieStorage::new(&path);
    let result = storage.set(REVEL_SESSION, input);

    println!();

    match result {
        Ok(_) => {
            success_ln!("Saved cookie at: {}", path.to_string_lossy().into_owned());
            Ok(Outcome::Success)
        }
        Err(e) => Err(e.into()),
    }
}

fn logout() -> Result<Outcome, AppError> {
    let path = get_cookie_path()?;

    if !path.exists() {
        warn_ln!("Cookie file does not exist.");
        return Ok(Outcome::Success);
    }

    std::fs::remove_file(&path).map_err(|source| CookieError::Remove {
        path: path.clone(),
        source,
    })?;

    success_ln!("Cookie removed.");
    Ok(Outcome::Success)
}

fn status() -> Result<Outcome, AppError> {
    let path = get_cookie_path()?;

    if !path.exists() || !path.is_file() {
        warn_ln!(
            "Cookie is not set. Run {} to log in to AtCoder.",
            styles::dim("`acc auth login`")
        );
        return Ok(Outcome::Success);
    }

    let storage = CookieStorage::new(path);
    let cookie = match storage.get(REVEL_SESSION) {
        Ok(Some(val)) => val,
        Ok(None) => {
            warn_ln!(
                "Necessary cookie ({}) does not exist. Run {} to log in to AtCoder.",
                REVEL_SESSION,
                styles::dim("`acc auth login`")
            );
            return Ok(Outcome::Success);
        }
        Err(e) => return Err(e.into()),
    };

    let spinner = Spinner::new("Testing cookie...");
    let client = AtCoderClient::new(Some(cookie));

    match client.fetch_page("https://atcoder.jp/contests/abc001/submit") {
        Ok(response) => {
            let status = response.status();

            if status.is_success() {
                spinner.finish(
                    &format!("{}", styles::success("200 OK")),
                    "Cookie is valid.",
                );
                Ok(Outcome::Success)
            } else if status.is_redirection() {
                spinner.finish(
                    &format!("{}", styles::error(&status.to_string())),
                    "Session expired. Please log in again.",
                );
                Ok(Outcome::Failed)
            } else {
                spinner.finish(
                    &format!("{}", styles::error(&status.to_string())),
                    "Received unexpected status code. Try again.",
                );
                Ok(Outcome::Failed)
            }
        }
        Err(e) => {
            let code = e
                .as_http()
                .and_then(|http| http.status())
                .map_or_else(|| String::from(ERROR_PREFIX), |s| s.to_string());
            spinner.finish(
                &format!("{}", styles::error(&code)),
                "A network error occurred:",
            );
            Err(e.into())
        }
    }
}
