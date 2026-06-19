use std::process::{ExitStatus, Stdio};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

const DEFAULT_TIMEOUT_SECS: u64 = 3;

/// Holds the result of a single test case execution.
pub struct RunResult {
    pub output: String,
    pub duration: Duration,
    pub memory_mb: Option<f64>,
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

/// Runs the executable with the given input, using GNU time if provided.
pub async fn run_case(
    run_cmd: &str,
    flags: &[String],
    input_data: &str,
    gnu_time: Option<&str>,
) -> Result<RunResult, RunError> {
    let time_limit = Duration::from_secs(DEFAULT_TIMEOUT_SECS);

    // Create a temporary file to store GNU time output if needed
    #[cfg(unix)]
    let gnu_time_file = match gnu_time {
        Some(_) => Some(tempfile::NamedTempFile::new().map_err(RunError::StartFailed)?),
        None => None,
    };

    let mut cmd_args = Vec::new();
    let mut program = run_cmd;

    #[cfg(unix)]
    if let Some(time_path) = gnu_time {
        let file_path = gnu_time_file.as_ref().unwrap().path().to_string_lossy().into_owned();
        cmd_args.extend_from_slice(&["-f".to_string(), "%M".to_string(), "-o".to_string(), file_path, "--".to_string()]);
        cmd_args.push(run_cmd.to_string());
        program = time_path;
    }

    cmd_args.extend_from_slice(flags);

    let start_time = Instant::now();

    let mut cmd = Command::new(program);
    cmd.args(&cmd_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            let _ = nix::unistd::setsid();
            Ok(())
        });
    }

    let mut child = cmd.spawn()?;
    let _pid = child.id();

    // Write to stdin asynchronously
    if let Some(mut stdin) = child.stdin.take() {
        let input_bytes = input_data.as_bytes().to_vec();
        tokio::spawn(async move {
            let _ = stdin.write_all(&input_bytes).await;
        });
    }

    // Wait for the process with a timeout
    let wait_future = child.wait_with_output();
    let result = timeout(time_limit, wait_future).await;

    let duration = start_time.elapsed();

    match result {
        Ok(Ok(output)) => {
            let mut final_status = output.status;
            let mut memory_mb = None;

            #[cfg(unix)]
            {
                if gnu_time.is_some() {
                    if let Some(code) = output.status.code() {
                        if code > 128 && code <= 128 + 64 {
                            final_status = ExitStatus::from_raw(code - 128);
                        }
                    }
                    if let Some(mut file) = gnu_time_file {
                        let mut content = String::new();
                        let _ = std::io::Read::read_to_string(&mut file, &mut content);
                        memory_mb = parse_memory_from_gnu_time(&content);
                    }
                }
            }

            if !final_status.success() {
                let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
                return Err(RunError::RuntimeError {
                    status: final_status,
                    stderr: stderr_str.trim().to_string(),
                });
            }

            let actual_output = String::from_utf8_lossy(&output.stdout).into_owned();
            Ok(RunResult {
                output: actual_output,
                duration,
                memory_mb,
            })
        }
        Ok(Err(e)) => Err(RunError::StartFailed(e)),
        Err(_) => {
            #[cfg(unix)]
            if let Some(child_pid) = _pid {
                let pgid = nix::unistd::Pid::from_raw(child_pid as i32);
                let _ = nix::sys::signal::killpg(pgid, nix::sys::signal::Signal::SIGKILL);
            }

            Err(RunError::Timeout { duration: time_limit })
        }
    }
}

/// Helper function to parse Max RSS (KB) from GNU time output and convert to MB.
#[cfg(unix)]
fn parse_memory_from_gnu_time(output: &str) -> Option<f64> {
    let last_line = output.lines().last()?;
    let trimmed = last_line.trim();
    if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(kb) = trimmed.parse::<f64>() {
            return Some(kb / 1024.0);
        }
    }
    None
}
