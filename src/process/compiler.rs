use std::process::{Command, Stdio};

/// Compiles the source code using the provided compiler command and flags.
pub fn compile(compiler: &str, flags: &[String]) -> Result<(), String> {
    let output = Command::new(compiler)
        .args(flags)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute compiler: {}", e))?;

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
        let err_msg = if stderr_str.trim().is_empty() {
            format!("Compilation failed with exit code: {}", output.status)
        } else {
            stderr_str
        };

        return Err(err_msg);
    }

    Ok(())
}
