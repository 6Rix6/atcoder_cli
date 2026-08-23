use crate::{
    config::{self},
    constants::REVEL_SESSION,
    cookie::{self, CookieError, storage::CookieStorage},
    error::AppError,
};

/// Outcome of looking up the REVEL_SESSION cookie.
#[derive(Debug)]
pub enum SessionStatus {
    /// The cookie is stored; contains its value.
    Available(String),
    /// No cookie has been saved yet.
    Missing,
    /// The cookie file exists but could not be read.
    Unreadable(CookieError),
}

/// Loads the REVEL_SESSION cookie without any printing.
pub fn get_session_status() -> Result<SessionStatus, AppError> {
    let cookie_path = cookie::utils::get_cookie_path()?;
    let storage = CookieStorage::new(&cookie_path);

    match storage.get(REVEL_SESSION) {
        Ok(Some(value)) => Ok(SessionStatus::Available(value)),
        Ok(None) => Ok(SessionStatus::Missing),
        Err(e) => Ok(SessionStatus::Unreadable(e)),
    }
}

/// Resolves the session cookie for a command that works anonymously without one.
///
/// Prints a warning when no cookie is set and fails when the cookie file is unreadable.
pub fn session_cookie() -> Result<Option<String>, AppError> {
    let path = cookie::utils::get_cookie_path()?;

    match get_session_status()? {
        SessionStatus::Available(value) => {
            success_ln!("Using Cookie in {}", path.display());
            Ok(Some(value))
        }
        SessionStatus::Missing => {
            warn_ln!("Cookie is not set.");
            Ok(None)
        }
        SessionStatus::Unreadable(e) => Err(e.into()),
    }
}

/// Loads the atcoder.toml config with local-first, global-fallback semantics.
///
/// A missing local file silently falls back to the global one, while a broken
/// local file warns before falling back. Returns `Ok(None)` when neither
/// exists.
pub fn load_config() -> Result<Option<config::models::Config>, AppError> {
    match config::utils::load_local_config() {
        Ok(Some(config)) => Ok(Some(config)),
        Ok(None) => Ok(config::utils::load_global_config()?),
        Err(e) => {
            warn_ln!("{}", e);
            warn_ln!("Continuing with global settings...");
            Ok(config::utils::load_global_config()?)
        }
    }
}
