use std::path::PathBuf;

use crate::constants::{CONFIG_DIR_NAME, COOKIE_FILE_NAME};

pub fn get_cookie_path() -> PathBuf {
    let mut config_path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_path.push(CONFIG_DIR_NAME);
    config_path.push(COOKIE_FILE_NAME);

    config_path
}

/// RFC 6265 cookie-octets: rejects empty values, whitespace, controls,
/// non-ASCII and characters that would break the `Cookie` header.
pub fn is_valid_session_value(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|b| {
            matches!(
                b,
                0x21 | 0x23..=0x2B | 0x2D..=0x3A | 0x3C..=0x5B | 0x5D..=0x7E
            )
        })
}
