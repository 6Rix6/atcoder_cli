use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file '{path}': {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse config file '{path}': {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("Failed to write config file '{path}': {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to serialize config file '{path}': {source}")]
    Serialize {
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },
}

