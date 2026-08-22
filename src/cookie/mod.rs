#![deny(clippy::unwrap_used, clippy::print_stdout, clippy::print_stderr)]

pub mod error;
pub mod storage;
pub mod utils;

pub use error::CookieError;
