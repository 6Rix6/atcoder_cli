use std::path::{Path, PathBuf};

use crate::{
    config::{error::ConfigError, models::Config},
    constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME},
};

pub fn load_local_config() -> Result<Option<Config>, ConfigError> {
    let local = PathBuf::from(CONFIG_FILE_NAME);
    load_config(&local)
}

pub fn load_global_config() -> Result<Option<Config>, ConfigError> {
    let Some(global) = dirs::config_dir().map(|d| d.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
    else {
        return Ok(None);
    };

    load_config(&global)
}

fn load_config(path: &Path) -> Result<Option<Config>, ConfigError> {
    if path.exists() {
        let toml_str = std::fs::read_to_string(path).map_err(|e| ConfigError::Read {
            path: path.to_path_buf(),
            source: e,
        })?;
        
        toml::from_str(&toml_str)
            .map(Some)
            .map_err(|e| ConfigError::Parse { 
                path: path.to_path_buf(), 
                source: e 
            })
    } else {
        Ok(None)
    }
}

pub fn create_config(config: Config, directory: &PathBuf) -> Result<PathBuf, ConfigError> {
    std::fs::create_dir_all(directory).map_err(|e| ConfigError::Write {
        path: directory.clone(),
        source: e,
    })?;

    let config_path = directory.join(CONFIG_FILE_NAME);

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
