use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use crate::cookie::error::CookieError;

/// Owner read/write only (0600); the file stores a session credential.
const COOKIE_FILE_MODE: u32 = 0o600;

pub struct CookieStorage {
    file_path: PathBuf,
}

impl CookieStorage {
    /// Initializes the storage with a specific file path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            file_path: path.as_ref().to_path_buf(),
        }
    }

    /// Helper to ensure the parent directories exist before writing.
    fn ensure_directory(&self) -> Result<(), CookieError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|source| CookieError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        Ok(())
    }

    /// Reads all cookies from the file into a HashMap.
    pub fn load(&self) -> Result<HashMap<String, String>, CookieError> {
        let mut cookies = HashMap::new();

        // If the file doesn't exist yet, return an empty map
        if !self.file_path.exists() {
            return Ok(cookies);
        }

        let file = File::open(&self.file_path).map_err(|source| CookieError::Read {
            path: self.file_path.clone(),
            source,
        })?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|source| CookieError::Read {
                path: self.file_path.clone(),
                source,
            })?;
            let trimmed = line.trim();

            // Skip empty lines or comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Split by the first '=' character
            if let Some((key, value)) = trimmed.split_once('=') {
                cookies.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(cookies)
    }

    /// Saves all cookies from a HashMap back into the file.
    pub fn save(&self, cookies: &HashMap<String, String>) -> Result<(), CookieError> {
        self.ensure_directory()?;
        let mut file = self.open_private_file()?;

        for (key, value) in cookies {
            writeln!(file, "{}={}", key, value).map_err(|source| CookieError::Write {
                path: self.file_path.clone(),
                source,
            })?;
        }

        Ok(())
    }

    /// Utility to get a single cookie value by key.
    pub fn get(&self, key: &str) -> Result<Option<String>, CookieError> {
        let cookies = self.load()?;
        Ok(cookies.get(key).cloned())
    }

    /// Utility to set or update a single cookie key=value.
    pub fn set(&self, key: &str, value: &str) -> Result<(), CookieError> {
        let mut cookies = self.load()?;
        cookies.insert(key.to_string(), value.to_string());
        self.save(&cookies)
    }

    /// Opens the cookie file for writing with owner-only permissions.
    fn open_private_file(&self) -> Result<File, CookieError> {
        #[cfg(unix)]
        {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(COOKIE_FILE_MODE)
                .open(&self.file_path)
                .map_err(|source| CookieError::Write {
                    path: self.file_path.clone(),
                    source,
                })?;

            // Harden the mode even when the file already existed with looser permissions.
            fs::set_permissions(
                &self.file_path,
                fs::Permissions::from_mode(COOKIE_FILE_MODE),
            )
            .map_err(|source| CookieError::Write {
                path: self.file_path.clone(),
                source,
            })?;

            Ok(file)
        }

        #[cfg(not(unix))]
        File::create(&self.file_path).map_err(|source| CookieError::Write {
            path: self.file_path.clone(),
            source,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn storage_in(dir: &TempDir) -> CookieStorage {
        CookieStorage::new(dir.path().join("cookies.txt"))
    }

    #[test]
    fn set_and_get_roundtrip() {
        let dir = TempDir::new().unwrap();
        let storage = storage_in(&dir);

        storage.set("REVEL_SESSION", "secret-value").unwrap();
        assert_eq!(
            storage.get("REVEL_SESSION").unwrap(),
            Some("secret-value".to_string())
        );

        // 上書き
        storage.set("REVEL_SESSION", "rotated").unwrap();
        assert_eq!(
            storage.get("REVEL_SESSION").unwrap(),
            Some("rotated".to_string())
        );

        // 未登録キー
        assert_eq!(storage.get("OTHER").unwrap(), None);
    }

    #[test]
    fn missing_file_reads_as_empty() {
        let dir = TempDir::new().unwrap();
        let storage = storage_in(&dir);

        assert_eq!(storage.get("REVEL_SESSION").unwrap(), None);
    }

    #[test]
    fn skips_comments_blank_lines_and_malformed_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cookies.txt");
        std::fs::write(
            &path,
            "# comment\n\n  \nA=1\ngarbage without equals\nB = 2 \n",
        )
        .unwrap();

        let storage = CookieStorage::new(&path);
        let cookies = storage.load().unwrap();

        assert_eq!(cookies.get("A").map(String::as_str), Some("1"));
        assert_eq!(cookies.get("B").map(String::as_str), Some("2"));
        assert_eq!(cookies.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn file_is_owner_only_after_save() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cookies.txt");

        // 緩い権限で事前作成しても保存時に0600へ締められる
        std::fs::write(&path, "REVEL_SESSION=old\n").unwrap();
        std::fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        let storage = CookieStorage::new(&path);
        storage.set("NEW", "value").unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        // 既存エントリは保持される
        assert_eq!(
            storage.get("REVEL_SESSION").unwrap(),
            Some("old".to_string())
        );
    }
}
