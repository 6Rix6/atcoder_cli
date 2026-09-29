use crate::{
    cli::event::{self, Level, CommonEvent},
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
            event::emit(&CommonEvent::Log {
                level: Level::Success,
                message: format!("Using Cookie in {}", path.display()),
            });
            Ok(Some(value))
        }
        SessionStatus::Missing => {
            event::emit(&CommonEvent::Log {
                level: Level::Warn,
                message: "Cookie is not set.".into(),
            });
            Ok(None)
        }
        SessionStatus::Unreadable(e) => Err(e.into()),
    }
}
