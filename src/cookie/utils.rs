use std::path::PathBuf;

use crate::constants::{CONFIG_DIR_NAME, COOKIE_FILE_NAME};

pub fn get_cookie_path() -> PathBuf {
    let mut config_path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_path.push(CONFIG_DIR_NAME);
    config_path.push(COOKIE_FILE_NAME);

    return config_path;
}



