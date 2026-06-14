use std::path::PathBuf;

use crate::{
    config::{error::ConfigError, models::Config},
    constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME}
};

pub fn load_local_config() -> Result<Option<Config>, ConfigError> {
    let local = PathBuf::from(CONFIG_FILE_NAME);
    load_config(local)
}

pub fn load_global_config() -> Result<Option<Config>, ConfigError> {
    // let local = PathBuf::from(CONFIG_FILE_NAME);
    // if local.exists() {
    //     let toml_str = std::fs::read_to_string(&local).map_err(|e| ConfigError::Read {
    //         path: local.clone(),
    //         source: e,
    //     })?;
    //
    //     match toml::from_str::<Config>(&toml_str) {
    //         Ok(config) => return Ok(Some(config)),
    //         Err(e) => {
    //             eprintln!(
    //                 "{}: Failed to parse {}: {}",
    //                 styles::warn("Warning"),
    //                 local.display(),
    //                 e
    //             );
    //             println!("Continuing with global settings...");
    //         }
    //     }
    // }

    let Some(global) = dirs::config_dir().map(|d| d.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
    else {
        return Ok(None);
    };

    load_config(global)
}

fn load_config(path: PathBuf) -> Result<Option<Config>, ConfigError> {
    if path.exists() {
        let toml_str = std::fs::read_to_string(&path).map_err(|e| ConfigError::Read {
            path: path.clone(),
            source: e,
        })?;
        toml::from_str(&toml_str)
            .map(Some)
            .map_err(|e| ConfigError::Parse { path, source: e })
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
