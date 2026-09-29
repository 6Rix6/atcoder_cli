use std::io::{Read, Write as _};
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

#[cfg(unix)]
use std::os::unix::process::{CommandExt, ExitStatusExt};

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
    RuntimeError {
        status: ExitStatus,
        stderr: String,
        duration: Duration,
    },
}

/// Runs the executable with the given input, using GNU time if provided.
///
/// The child runs in its own session (unix); a watchdog thread kills the whole
/// group when `time_limit_secs` elapses, which turns into `RunError::Timeout`.
pub fn run_case(
    run_cmd: &str,
    flags: &[String],
    input_data: &str,
    gnu_time: Option<&str>,
    time_limit_secs: u32,
) -> Result<RunResult, RunError> {
    let time_limit = Duration::from_secs(u64::from(time_limit_secs.max(1)));
    let start_time = Instant::now();

    // Create a temporary file to store GNU time output if needed
    #[cfg(unix)]
    let gnu_time_file = match gnu_time {
        Some(_) => Some(tempfile::NamedTempFile::new().map_err(RunError::StartFailed)?),
        None => None,
    };

    let mut cmd_args: Vec<String> = Vec::new();
    let mut program = run_cmd;

    #[cfg(unix)]
    if let Some(time_path) = gnu_time
        && let Some(file) = gnu_time_file.as_ref()
    {
        let file_path = file.path().to_string_lossy().into_owned();
        cmd_args.extend([
            "-f".to_string(),
            "%M".to_string(),
            "-o".to_string(),
            file_path,
            "--".to_string(),
        ]);
        cmd_args.push(run_cmd.to_string());
        program = time_path;
    }

    cmd_args.extend(flags.iter().cloned());

    let mut command = std::process::Command::new(program);
    command
        .args(&cmd_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            let _ = nix::unistd::setsid();
            Ok(())
        });
    }

    let mut child = command.spawn()?;
    let pid = child.id();

    // Feed stdin from a thread; the child may exit before reading everything.
    let stdin_writer = child.stdin.take().map(|mut stdin| {
        let input = input_data.as_bytes().to_vec();
        thread::spawn(move || {
            let _ = stdin.write_all(&input);
        })
    });

    // Watchdog: kill the process group once the channel goes silent for the
    // full time limit. Dropping `done_tx` after the wait disconnects it.
    let timed_out = Arc::new(AtomicBool::new(false));
    let (done_tx, done_rx) = mpsc::channel::<()>();

    let watchdog = {
        let timed_out = Arc::clone(&timed_out);
        thread::spawn(move || match done_rx.recv_timeout(time_limit) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => {
                timed_out.store(true, Ordering::SeqCst);
                #[cfg(unix)]
                {
                    use nix::sys::signal::{Signal, killpg};
                    use nix::unistd::Pid;

                    let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
                }
            }
        })
    };

    let waited = child.wait_with_output();

    drop(done_tx);
    if let Some(handle) = stdin_writer {
        let _ = handle.join();
    }
    let _ = watchdog.join();

    let output = waited?;

    let mut final_status = output.status;
    let mut memory_mb = None;

    #[cfg(unix)]
    if gnu_time.is_some() {
        // GNU time reports the wrapped command's signal exit as 128+signal.
        if let Some(code) = output.status.code()
            && (129..=192).contains(&code)
        {
            final_status = ExitStatus::from_raw(code - 128);
        }
        if let Some(mut file) = gnu_time_file {
            let mut content = String::new();
            let _ = file.read_to_string(&mut content);
            memory_mb = parse_memory_from_gnu_time(&content);
        }
    }

    if timed_out.load(Ordering::SeqCst) {
        return Err(RunError::Timeout {
            duration: start_time.elapsed(),
        });
    }

    if !final_status.success() {
        return Err(RunError::RuntimeError {
            status: final_status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            duration: start_time.elapsed(),
        });
    }

    Ok(RunResult {
        output: String::from_utf8_lossy(&output.stdout).into_owned(),
        duration: start_time.elapsed(),
        memory_mb,
    })
}

/// Helper function to parse Max RSS (KB) from GNU time output and convert to MB.
#[cfg(unix)]
fn parse_memory_from_gnu_time(output: &str) -> Option<f64> {
    let last_line = output.lines().last()?;
    let trimmed = last_line.trim();
    if !trimmed.is_empty()
        && trimmed.chars().all(|c| c.is_ascii_digit())
        && let Ok(kb) = trimmed.parse::<f64>()
    {
        return Some(kb / 1024.0);
    }
    None
}

#[cfg(all(test, unix))]
mod tests {
    use super::parse_memory_from_gnu_time;

    #[test]
    fn parses_last_line_as_kib() {
        assert_eq!(parse_memory_from_gnu_time("4096\n"), Some(4.0));
        // GNU time -f はコマンド出力の後に値の行を出力する
        assert_eq!(
            parse_memory_from_gnu_time("program stdout\n8192"),
            Some(8.0)
        );
    }

    #[test]
    fn rejects_non_numeric_output() {
        assert_eq!(parse_memory_from_gnu_time(""), None);
        assert_eq!(parse_memory_from_gnu_time("\n"), None);
        assert_eq!(parse_memory_from_gnu_time("not a number"), None);
        assert_eq!(parse_memory_from_gnu_time("12.5"), None);
        assert_eq!(parse_memory_from_gnu_time("-1\n"), None);
    }
}
