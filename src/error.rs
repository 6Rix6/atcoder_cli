use thiserror::Error;

use crate::{
    atcoder::AtcoderError, config::error::ConfigError, cookie::CookieError,
    process::compiler::CompileError,
};

/// Top-level error for command execution.
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Atcoder(#[from] AtcoderError),

    #[error(transparent)]
    Cookie(#[from] CookieError),

    #[error(transparent)]
    Compile(#[from] CompileError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Prompt(#[from] dialoguer::Error),

    #[error("invalid task URL or ID: '{0}'")]
    InvalidTask(String),

    #[error("no config file found")]
    NoConfig,
}
