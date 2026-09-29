use std::{collections::BTreeMap, time::Duration};

use clap::Args;
use serde::Serialize;

use crate::{
    atcoder::{
        self,
        client::AtCoderClient,
        models::{SampleInput, TaskData},
    },
    cli::{
        Outcome,
        event::{self, CliEvent, CommonEvent, Level, Task},
        styles,
        template::expand_template,
        utils,
    },
    error::AppError,
    process::{
        compiler,
        runner::{self, RunError},
    },
    utils::gnu_time::get_gnu_time_path,
};

/// Run sample test cases for a task.
#[derive(Debug, Args)]
pub struct TestArgs {
    /// Task ID or problem URL (e.g. "abc350_a" or the tasks/... URL)
    pub task_id: String,
}

#[derive(Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Verdict {
    Ac,
    Wa,
    Re,
    Tle,
}

#[derive(Serialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum TestEvent {
    TestStarted {
        task_id: String,
        url: String,
    },
    TestFinished {
        total: u32,
        passed: u32,
        submit_url: String,
    },
    TestCaseStarted {
        index: u32,
    },
    TestCaseResult {
        index: u32,
        verdict: Verdict,
        #[serde(with = "utils::serde_format::duration_ms_f64", rename = "time_ms")]
        time: Duration,
        #[serde(skip_serializing_if = "Option::is_none")]
        memory_mb: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        actual: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        log: Option<String>,
    },
    TestCaseFailed {
        error: String,
    },
    CompileStarted {
        command: String,
    },
    CompileFinished {
        success: bool,
    },
}

impl CliEvent for TestEvent {
    fn print_human(&self) {
        match self {
            TestEvent::TestStarted { task_id, .. } => {
                success_ln!("Running for task: {}", styles::success(task_id));
            }
            TestEvent::TestFinished {
                total,
                passed,
                submit_url,
            } => {
                if total == passed {
                    success_ln!("Test {}", styles::success("succeeded"));
                    info_ln!("Submit here: {}", submit_url);
                } else {
                    failed_ln!(
                        "Test {}: {}/{} passed",
                        styles::error("failed"),
                        passed,
                        total
                    );
                }
            }
            TestEvent::TestCaseStarted { index } => {
                info_ln!("Sample #{}", index);
            }
            TestEvent::TestCaseResult {
                verdict: Verdict::Ac,
                time,
                memory_mb,
                ..
            } => {
                info_ln!("time: {}", styles::dim(&format!("{:?}", time)));
                if let Some(data) = memory_mb {
                    info_ln!("memory: {}", styles::dim(&format!("{:.3}mb", data)));
                }
                success_ln!("{}", styles::success("AC"));
            }
            TestEvent::TestCaseResult {
                verdict: Verdict::Wa,
                time,
                memory_mb,
                expected,
                actual,
                ..
            } => {
                info_ln!("time: {}", styles::dim(&format!("{:?}", time)));
                if let Some(data) = memory_mb {
                    info_ln!("memory: {}", styles::dim(&format!("{:.3}mb", data)));
                }
                failed_ln!("{}", styles::warn("WA"));
                println!(
                    "{}:\n{}",
                    styles::dim("Expected"),
                    expected.as_deref().unwrap_or_default()
                );
                println!(
                    "{}:\n{}",
                    styles::dim("Actual"),
                    actual.as_deref().unwrap_or_default()
                );
            }
            TestEvent::TestCaseResult {
                verdict: Verdict::Tle,
                time,
                ..
            } => {
                info_ln!("time: {}", styles::dim(&format!("{:?}", time)));
                failed_ln!("{} (Time Limit Exceeded)", styles::error("TLE"));
            }
            TestEvent::TestCaseResult {
                verdict: Verdict::Re,
                exit_code,
                log,
                ..
            } => {
                failed_ln!("{} (Runtime Error)", styles::error("RE"));
                println!(
                    "{}:\n  {}",
                    styles::dim("Exit Code"),
                    exit_code.as_deref().unwrap_or_default()
                );
                let log_text = log.as_deref().unwrap_or_default();
                if !log_text.is_empty() {
                    println!("{}:\n  {}", styles::dim("Log"), log_text);
                }
            }
            TestEvent::TestCaseFailed { .. } => {
                error_ln!("System Error (Could not run the binary)");
            }
            TestEvent::CompileStarted { .. } | TestEvent::CompileFinished { .. } => {}
        }
    }
}

/// Handler for 'acc test <task_id>'
pub fn run(args: &TestArgs) -> Result<Outcome, AppError> {
    let Some(task_info) = atcoder::utils::get_task_info(&args.task_id) else {
        return Err(AppError::InvalidTask(args.task_id.clone()));
    };

    event::emit(&TestEvent::TestStarted {
        task_id: task_info.task_id.clone(),
        url: task_info.url.clone(),
    });

    let config = utils::load_config()?.ok_or(AppError::NoConfig)?;

    let vars = task_info.to_map();

    if let Some(compiler_template) = &config.compiler {
        let flags = config.compiler_flags.as_deref().unwrap_or_default();
        compile(compiler_template, flags, &vars)?;
    }

    let cookie = utils::cookie::session_cookie()?;
    let data = fetch_task_data(&task_info.url, cookie)?;

    if data.samples.is_empty() {
        return Err(atcoder::AtcoderError::NoSamples {
            url: task_info.url.clone(),
        }
        .into());
    }

    event::emit_empty_ln();

    let gnu_time_path = get_gnu_time_path();
    if gnu_time_path.is_none() {
        event::emit(&CommonEvent::Log {
            level: Level::Warn,
            message: String::from("GNU time is not available. Memory usage will not be measured."),
        });
    }

    event::emit_empty_ln();

    let mut passed = 0;
    let run_command = expand_template(&config.run, &vars);
    let run_flags: Vec<String> = config
        .run_flags
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|f| expand_template(f, &vars))
        .collect();

    for (i, sample) in data.samples.iter().enumerate() {
        if execute_test_case(
            i + 1,
            &run_command,
            &run_flags,
            sample,
            gnu_time_path.as_ref().and_then(|p| p.to_str()),
            data.page.time_limit,
        )? {
            passed += 1;
        }
    }

    let total: u32 = data.samples.len() as u32;
    event::emit(&TestEvent::TestFinished {
        total,
        passed,
        submit_url: String::from(&task_info.submit_url),
    });

    if passed == total {
        Ok(Outcome::Success)
    } else {
        Ok(Outcome::Failed)
    }
}

fn compare_lines_trimmed(s1: &str, s2: &str) -> bool {
    s1.lines()
        .map(|l| l.trim())
        .eq(s2.lines().map(|l| l.trim()))
}

fn compile(
    compiler_template: &str,
    compiler_flags: &[String],
    vars: &BTreeMap<&str, &str>,
) -> Result<(), AppError> {
    let compiler_command = expand_template(compiler_template, vars);
    let compiler_flags: Vec<String> = compiler_flags
        .iter()
        .map(|f| expand_template(f, vars))
        .collect();

    let task = Task::start(
        &TestEvent::CompileStarted {
            command: format!("{} {}", compiler_command, compiler_flags.join(" ")),
        },
        "Compiling source...",
    );

    match compiler::compile(&compiler_command, &compiler_flags) {
        Ok(()) => {
            task.finish_ok(
                &TestEvent::CompileFinished { success: true },
                None,
                "Compile finished.",
            );
            Ok(())
        }
        Err(e) => {
            task.finish_err(
                &TestEvent::CompileFinished { success: false },
                Some("CE"),
                "Failed to compile source.",
            );
            Err(e.into())
        }
    }
}

fn fetch_task_data(url: &str, cookie: Option<String>) -> Result<TaskData, AppError> {
    let client = AtCoderClient::new(cookie);
    let task = Task::start(
        &CommonEvent::FetchStarted {
            url: String::from(url),
        },
        "Fetching samples...",
    );

    match client.get_task(url) {
        Ok(val) => {
            task.finish_ok(
                &CommonEvent::FetchFinished {
                    success: true,
                    status_code: Some(String::from("200")),
                },
                Some("200 OK"),
                &format!("{} sample(s) fetched.", val.samples.len()),
            );
            Ok(val)
        }
        Err(e) => {
            let code = e
                .as_http()
                .and_then(|http| http.status())
                .map(|s| s.to_string());

            task.finish_err(
                &CommonEvent::FetchFinished {
                    success: false,
                    status_code: code.clone(),
                },
                code.as_deref(),
                "Failed to fetch samples.",
            );

            Err(e.into())
        }
    }
}

/// Runs one sample and prints its verdict. Returns `Ok(true)` on AC.
///
/// `Err` is reserved for system failures that abort the whole run.
fn execute_test_case(
    case_num: usize,
    run_command: &str,
    run_flags: &[String],
    sample: &SampleInput,
    gnu_time: Option<&str>,
    time_limit_secs: u32,
) -> Result<bool, AppError> {
    event::emit(&TestEvent::TestCaseStarted {
        index: case_num as u32,
    });
    let mut is_ac = false;

    let result_event: TestEvent = match runner::run_case(
        run_command,
        run_flags,
        &sample.input,
        gnu_time,
        time_limit_secs,
    ) {
        Ok(result) => {
            let verdict = if compare_lines_trimmed(&result.output, &sample.output) {
                Verdict::Ac
            } else {
                Verdict::Wa
            };
            is_ac = verdict == Verdict::Ac;

            TestEvent::TestCaseResult {
                index: case_num as u32,
                verdict,
                time: result.duration,
                memory_mb: result.memory_mb,
                expected: Some(sample.output.clone()),
                actual: Some(result.output),
                exit_code: None,
                log: None,
            }
        }
        Err(RunError::Timeout { duration }) => {
            TestEvent::TestCaseResult {
                index: case_num as u32,
                verdict: Verdict::Tle,
                time: duration,
                memory_mb: None,
                expected: Some(sample.output.clone()),
                actual: None,
                exit_code: None,
                log: None,
            }
        }
        Err(RunError::RuntimeError {
            status,
            stderr,
            duration,
        }) => {
            TestEvent::TestCaseResult {
                index: case_num as u32,
                verdict: Verdict::Re,
                time: duration,
                memory_mb: None,
                expected: Some(sample.output.clone()),
                actual: None,
                exit_code: Some(status.to_string()),
                log: Some(stderr),
            }
        }
        Err(RunError::StartFailed(source)) => {
            event::emit(&TestEvent::TestCaseFailed {
                error: source.to_string(),
            });

            return Err(source.into());
        }
    };

    event::emit(&result_event);
    event::emit_empty_ln();
    Ok(is_ac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_lines_ignoring_whitespace() {
        assert!(compare_lines_trimmed("1 2\n3", "1 2\n3"));
        assert!(compare_lines_trimmed("1 2  \n\t3 \n", "1 2\n3"));
        assert!(compare_lines_trimmed("a\nb", "a\nb\n"));
        assert!(!compare_lines_trimmed("a\nb", "a\nc"));
        assert!(!compare_lines_trimmed("a\nb", "ab"));
    }
}
