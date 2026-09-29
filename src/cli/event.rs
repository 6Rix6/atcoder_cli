use std::{
    marker::PhantomData,
    sync::{LazyLock, OnceLock},
};

use serde::Serialize;

use crate::cli::styles::{self, ERROR_PREFIX, SUCCESS_PREFIX, spinner::Spinner};

static DEFAULT_IS_JSON: LazyLock<bool> = LazyLock::new(|| false);
static IS_JSON: OnceLock<bool> = OnceLock::new();

pub fn set_is_json(value: bool) -> Result<(), bool> {
    IS_JSON.set(value)
}

fn is_json() -> bool {
    *IS_JSON.get().unwrap_or(&DEFAULT_IS_JSON)
}

pub trait CliEvent: Serialize {
    fn print_human(&self);
}

pub fn emit<E: CliEvent>(event: &E) {
    if is_json() {
        match serde_json::to_string(event) {
            Ok(line) => println!("{line}"),
            Err(e) => eprintln!("failed to serialize event: {e}"),
        }
    } else {
        event.print_human();
    }
}

pub fn emit_empty_ln() {
    if !is_json() {
        println!();
    }
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Success,
    Info,
    Warn,
}

#[derive(Serialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum CommonEvent {
    Log {
        level: Level,
        message: String,
    },
    Error {
        message: String,
    },

    // For `Task`; no print_human output.
    FetchStarted {
        url: String,
    },
    FetchFinished {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        status_code: Option<String>,
    },
}

impl CliEvent for CommonEvent {
    fn print_human(&self) {
        match self {
            CommonEvent::Log {
                level: Level::Success,
                message,
            } => {
                success_ln!("{}", message);
            }
            CommonEvent::Log {
                level: Level::Warn,
                message,
            } => {
                warn_ln!("{}", message);
            }
            CommonEvent::Log {
                level: Level::Info,
                message,
            } => {
                info_ln!("{}", message);
            }
            CommonEvent::Error { message } => {
                error_ln!("{}", message);
            }
            _ => {}
        }
    }
}

pub struct Task<E: CliEvent> {
    spinner: Option<Spinner>,
    _marker: PhantomData<E>,
}

impl<E: CliEvent> Task<E> {
    pub fn start(event: &E, spinner_msg: &str) -> Self {
        if is_json() {
            emit(event);
            Self {
                spinner: None,
                _marker: PhantomData,
            }
        } else {
            Self {
                spinner: Some(Spinner::new(spinner_msg)),
                _marker: PhantomData,
            }
        }
    }

    pub fn finish_ok(self, event: &E, spinner_prefix: Option<&str>, spinner_msg: &str) {
        match self.spinner {
            Some(sp) => sp.finish(
                &format!(
                    "{}",
                    styles::success(spinner_prefix.unwrap_or(SUCCESS_PREFIX))
                ),
                spinner_msg,
            ),
            None => emit(event),
        }
    }

    pub fn finish_err(self, event: &E, spinner_prefix: Option<&str>, spinner_msg: &str) {
        match self.spinner {
            Some(sp) => sp.finish(
                &format!("{}", styles::error(spinner_prefix.unwrap_or(ERROR_PREFIX))),
                spinner_msg,
            ),
            None => emit(event),
        }
    }
}
