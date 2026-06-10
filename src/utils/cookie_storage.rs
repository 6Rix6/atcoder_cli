use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn get_cookie_path() -> PathBuf {
    let mut config_path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_path.push("atcoder-cli");
    config_path.push("cookies.txt");

    return config_path;
}


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
    fn ensure_directory(&self) -> io::Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Reads all cookies from the file into a HashMap.
    pub fn load(&self) -> io::Result<HashMap<String, String>> {
        let mut cookies = HashMap::new();

        // If the file doesn't exist yet, return an empty map
        if !self.file_path.exists() {
            return Ok(cookies);
        }

        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
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
    pub fn save(&self, cookies: &HashMap<String, String>) -> io::Result<()> {
        self.ensure_directory()?;
        let mut file = File::create(&self.file_path)?;

        for (key, value) in cookies {
            writeln!(file, "{}={}", key, value)?;
        }

        Ok(())
    }

    /// Utility to get a single cookie value by key.
    pub fn get(&self, key: &str) -> io::Result<Option<String>> {
        let cookies = self.load()?;
        Ok(cookies.get(key).cloned())
    }

    /// Utility to set or update a single cookie key=value.
    pub fn set(&self, key: &str, value: &str) -> io::Result<()> {
        let mut cookies = self.load()?;
        cookies.insert(key.to_string(), value.to_string());
        self.save(&cookies)
    }
}
