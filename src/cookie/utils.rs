use std::path::PathBuf;

use crate::constants::{CONFIG_DIR_NAME, COOKIE_FILE_NAME};
use crate::cookie::error::CookieError;

pub fn get_cookie_path() -> Result<PathBuf, CookieError> {
    let config_dir = dirs::config_dir().ok_or(CookieError::NoConfigDir)?;

    Ok(config_dir.join(CONFIG_DIR_NAME).join(COOKIE_FILE_NAME))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_typical_session_values() {
        assert!(is_valid_session_value("abcXYZ123"));
        assert!(is_valid_session_value("with-dash_under.dot~plus+eq=sign"));
        assert!(is_valid_session_value("%2Fencoded%20value"));
        assert!(is_valid_session_value("a"));
    }

    #[test]
    fn rejects_header_breaking_values() {
        assert!(!is_valid_session_value(""));
        assert!(!is_valid_session_value("has space"));
        assert!(!is_valid_session_value("semi;colon"));
        assert!(!is_valid_session_value("com,ma"));
        assert!(!is_valid_session_value("quo\"te"));
        assert!(!is_valid_session_value("back\\slash"));
        // 改行・制御文字はヘッダインジェクションになる
        assert!(!is_valid_session_value("new\nline"));
        assert!(!is_valid_session_value("tab\there"));
        assert!(!is_valid_session_value("\u{7f}del"));
    }

    #[test]
    fn rejects_non_ascii() {
        assert!(!is_valid_session_value("日本語"));
        assert!(!is_valid_session_value("café"));
    }
}
