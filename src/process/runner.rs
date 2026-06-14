use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Holds the result of a single test case execution.
pub struct RunResult {
    pub output: String,
    pub duration: Duration,
}

/// Runs the executable with the given input and returns the result.
pub fn run_case(run_cmd: &str, flags: &[String], input_data: &str) -> Result<RunResult, String> {
    let start_time = Instant::now();

    let mut process = Command::new(run_cmd)
        .args(flags)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start process: {}", e))?;

    if let Some(mut stdin) = process.stdin.take() {
        stdin
            .write_all(input_data.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    }

    let output = process
        .wait_with_output()
        .map_err(|e| format!("Failed to get output: {}", e))?;

    let duration = start_time.elapsed();

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
        
        let mut err_msg = format!("Exit status: {}", output.status);
        if !stderr_str.trim().is_empty() {
            err_msg.push_str("\n");
            err_msg.push_str(stderr_str.trim());
        }
        
        return Err(err_msg);
    }

    let actual_output = String::from_utf8_lossy(&output.stdout).into_owned();

    Ok(RunResult {
        output: actual_output,
        duration,
    })
}
