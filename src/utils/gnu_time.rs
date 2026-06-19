use std::path::PathBuf;

/// Determine if GNU time is available, and if so, return its absolute path.
pub fn get_gnu_time_path() -> Option<PathBuf> {
    if cfg!(windows) {
        return None;
    }

    let candidates = ["/usr/bin/time", "gtime", "time"];

    for cmd in candidates {
        if let Ok(path) = which::which(cmd) {
            if is_real_gnu_time(&path) {
                return Some(path);
            }
        }
    }

    None
}

/// Check if the time command found is actually "GNU time" and not a shell built-in or something else.
pub fn is_real_gnu_time(path: &std::path::Path) -> bool {
    use std::process::Command;
    
    if let Ok(output) = Command::new(path).arg("--version").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("GNU");
        }
    }
    false
}
