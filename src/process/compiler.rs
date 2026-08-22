use std::io;
use std::process::{Command, ExitStatus, Stdio};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("failed to execute compiler `{program}`")]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },

    #[error("compilation failed with {status}")]
    Failed { status: ExitStatus },

    #[error("compilation failed with {status}:\n{stderr}")]
    FailedWithOutput { status: ExitStatus, stderr: String },
}

/// Compiles the source code using the provided compiler command and flags.
pub fn compile(compiler: &str, flags: &[String]) -> Result<(), CompileError> {
    let output = Command::new(compiler)
        .args(flags)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|source| CompileError::Spawn {
            program: compiler.to_string(),
            source,
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();

        return Err(if stderr.is_empty() {
            CompileError::Failed {
                status: output.status,
            }
        } else {
            CompileError::FailedWithOutput {
                status: output.status,
                stderr: stderr.to_string(),
            }
        });
    }

    Ok(())
}
