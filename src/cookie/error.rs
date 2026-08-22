use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CookieError {
    #[error("failed to read cookie file '{path}'")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write cookie file '{path}'")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to remove cookie file '{path}'")]
    Remove {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not determine config directory")]
    NoConfigDir,
}
