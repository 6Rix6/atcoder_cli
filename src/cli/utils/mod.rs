use crate::{
    cli::event::{self, CommonEvent, Level},
    config::{self},
    error::AppError,
};

pub mod cookie;
pub mod serde_format;

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
            event::emit(&CommonEvent::Log {
                level: Level::Warn,
                message: format!("{}", e),
            });
            event::emit(&CommonEvent::Log {
                level: Level::Warn,
                message: String::from("Continuing with global settings..."),
            });

            Ok(config::utils::load_global_config()?)
        }
    }
}
