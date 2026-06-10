use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

use crate::constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub compiler: Option<String>,
    pub compiler_flags: Option<Vec<String>>,
    pub run: String,
    pub run_flags: Option<Vec<String>>,
    pub cookie: Option<String>,
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
