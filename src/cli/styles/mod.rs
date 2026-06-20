use console::{StyledObject, style};

pub fn error<D>(msg: D) -> StyledObject<D> {
    style(msg).for_stderr().red().bold()
}

pub fn warn<D>(msg: D) -> StyledObject<D> {
    style(msg).yellow().bold()
}

pub fn success<D>(msg: D) -> StyledObject<D> {
    style(msg).green().bold().bright()
}

pub fn info<D>(msg: D) -> StyledObject<D> {
    style(msg).blue().bold().bright()
}

pub fn dim<D>(msg: D) -> StyledObject<D> {
    style(msg).dim()
}

pub const SUCCESS_PREFIX: &str = "✓";
pub const ERROR_PREFIX: &str = "ERROR";
pub const FAILED_PREFIX: &str = "-";
pub const WARN_PREFIX: &str = "!";
pub const INFO_PREFIX: &str = "*";

#[macro_use]
pub mod macros;

pub mod spinner;
pub mod dialog_theme;
