use std::process::{ExitStatus, Stdio};
use std::time::{Duration, Instant};
use thiserror::Error;
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

#[derive(Error, Debug)]
pub enum RunError {
    #[error("Failed to start process: {0}")]
    StartFailed(#[from] std::io::Error),

    #[error("Process timed out (TLE). Duration: {duration:?}")]
    Timeout { duration: Duration },

    #[error("Process exited with non-zero status: {status}\n{stderr}")]
    RuntimeError { status: ExitStatus, stderr: String },
}

/// Runs the executable asymmetrically with the given input and returns the result.
pub async fn run_case(
    run_cmd: &str,
    flags: &[String],
    input_data: &str,
) -> Result<RunResult, RunError> {
    let time_limit = Duration::from_secs(DEFAULT_TIMEOUT_SECS);
    let start_time = Instant::now();

    let mut child = Command::new(run_cmd)
        .args(flags)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

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
                return Err(RunError::RuntimeError {
                    status: output.status,
                    stderr: stderr_str.trim().to_string(),
                });
            }

            let actual_output = String::from_utf8_lossy(&output.stdout).into_owned();
            Ok(RunResult {
                output: actual_output,
                duration,
                is_timeout: false,
            })
        }
        Ok(Err(e)) => Err(RunError::StartFailed(e)),
        Err(_) => Err(RunError::Timeout {
            duration: time_limit,
        }),
    }
}
