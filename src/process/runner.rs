use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

const DEFAULT_TIMEOUT_SECS: u64 = 3;

/// Holds the result of a single test case execution.
pub struct RunResult {
    pub output: String,
    pub duration: Duration,
    pub is_timeout: bool,
}

/// Runs the executable asymmetrically with the given input and returns the result.
pub async fn run_case(
    run_cmd: &str,
    flags: &[String],
    input_data: &str,
) -> Result<RunResult, String> {
    let time_limit = Duration::from_secs(DEFAULT_TIMEOUT_SECS);
    let start_time = Instant::now();

    let mut child = Command::new(run_cmd)
        .args(flags)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to start process: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        let input_bytes = input_data.as_bytes().to_vec();
        tokio::spawn(async move {
            let _ = stdin.write_all(&input_bytes).await;
        });
    }

    let wait_future = child.wait_with_output();
    let result = timeout(time_limit, wait_future).await;

    let duration = start_time.elapsed();

    match result {
        Ok(Ok(output)) => {
            if !output.status.success() {
                let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
                let mut err_msg = format!("Exit status: {}", output.status);
                if !stderr_str.trim().is_empty() {
                    err_msg.push('\n');
                    err_msg.push_str(stderr_str.trim());
                }
                return Err(err_msg);
            }

            let actual_output = String::from_utf8_lossy(&output.stdout).into_owned();
            Ok(RunResult {
                output: actual_output,
                duration,
                is_timeout: false,
            })
        }
        Ok(Err(e)) => Err(format!("Failed to get output: {}", e)),
        Err(_) => {
            // Timeout occurred!
            Err("Process timed out. (TLE)".to_string())
        }
    }
}
