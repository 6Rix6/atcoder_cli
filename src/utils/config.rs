use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

use crate::constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub compiler: Option<String>,
    pub compiler_flags: Option<Vec<String>>,
    pub run: String,
    pub run_flags: Option<Vec<String>>,
    pub cookie: Option<String>,
}

pub struct ConfigRef {
    pub compiler: Option<&'static str>,
    pub compiler_flags: Option<&'static [&'static str]>,
    pub run: &'static str,
    pub run_flags: Option<&'static [&'static str]>,
    pub cookie: Option<&'static str>,
}

impl From<&ConfigRef> for Config {
    fn from(r: &ConfigRef) -> Self {
        Config {
            compiler: r.compiler.map(str::to_string),
            compiler_flags: r.compiler_flags.map(|f| f.iter().map(|s| s.to_string()).collect()),
            run: r.run.to_string(),
            run_flags: r.run_flags.map(|f| f.iter().map(|s| s.to_string()).collect()),
            cookie: r.cookie.map(str::to_string),
        }
    }
}

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

pub fn load_config() -> Result<Option<Config>, ConfigError> {
    let local = PathBuf::from(CONFIG_FILE_NAME);
    if local.exists() {
        let toml_str = std::fs::read_to_string(&local).map_err(|e| ConfigError::Read {
            path: local.clone(),
            source: e,
        })?;

        match toml::from_str::<Config>(&toml_str) {
            Ok(config) => return Ok(Some(config)),
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", local.display(), e);
                println!("Continuing with global settings...");
            }
        }
    }

    let Some(global) = dirs::config_dir().map(|d| d.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
    else {
        return Ok(None);
    };

    if global.exists() {
        let toml_str = std::fs::read_to_string(&global).map_err(|e| ConfigError::Read {
            path: global.clone(),
            source: e,
        })?;
        toml::from_str(&toml_str)
            .map(Some)
            .map_err(|e| ConfigError::Parse {
                path: global,
                source: e,
            })
    } else {
        Ok(None)
    }
}

pub fn create_config(config: Config) -> Result<PathBuf, ConfigError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ConfigError::Write {
            path: PathBuf::from(CONFIG_DIR_NAME),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            ),
        })?
        .join(CONFIG_DIR_NAME);

    std::fs::create_dir_all(&config_dir).map_err(|e| ConfigError::Write {
        path: config_dir.clone(),
        source: e,
    })?;

    let config_path = config_dir.join(CONFIG_FILE_NAME);

    let content = toml::to_string_pretty(&config).map_err(|e| ConfigError::Serialize {
        path: config_path.clone(),
        source: e,
    })?;

    std::fs::write(&config_path, content).map_err(|e| ConfigError::Write {
        path: config_path.clone(),
        source: e,
    })?;

    Ok(config_path)
}
