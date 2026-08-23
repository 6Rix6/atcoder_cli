use std::collections::BTreeMap;

use clap::Args;

use crate::{
    atcoder::{
        self,
        client::AtCoderClient,
        models::{SampleInput, TaskData},
    },
    cli::{
        Outcome,
        styles::{self, ERROR_PREFIX, SUCCESS_PREFIX, spinner::Spinner},
        template::expand_template,
        utils,
    },
    config::models::Config,
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

/// Handler for 'acc test <task_id>'
pub fn run(args: &TestArgs) -> Result<Outcome, AppError> {
    let Some(task_info) = atcoder::utils::get_task_info(&args.task_id) else {
        return Err(AppError::InvalidTask(args.task_id.clone()));
    };

    success_ln!("Running for task: {}", styles::success(&task_info.task_id));

    let config = load_config()?.ok_or(AppError::NoConfig)?;

    let vars = task_info.to_map();

    if let Some(compiler_template) = &config.compiler {
        let flags = config.compiler_flags.as_deref().unwrap_or_default();
        compile(compiler_template, flags, &vars)?;
    }

    let cookie = utils::session_cookie()?;
    let data = fetch_task_data(&task_info.url, cookie)?;

    if data.samples.is_empty() {
        return Err(atcoder::AtcoderError::NoSamples {
            url: task_info.url.clone(),
        }
        .into());
    }

    println!();

    let gnu_time_path = get_gnu_time_path();
    if gnu_time_path.is_none() {
        warn_ln!("GNU time is not available. Memory usage will not be measured.");
    }

    println!();

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

    if passed == data.samples.len() {
        success_ln!("Test {}", styles::success("succeeded"));
        info_ln!("Submit here: {}", &task_info.submit_url);
        Ok(Outcome::Success)
    } else {
        failed_ln!(
            "Test {}: {}/{} passed",
            styles::error("failed"),
            passed,
            data.samples.len()
        );
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
    let spinner = Spinner::new("Compiling source...");

    let compiler_command = expand_template(compiler_template, vars);
    let compiler_flags: Vec<String> = compiler_flags
        .iter()
        .map(|f| expand_template(f, vars))
        .collect();

    match compiler::compile(&compiler_command, &compiler_flags) {
        Ok(()) => {
            spinner.finish(
                &format!("{}", styles::success(SUCCESS_PREFIX)),
                "Compile finished.",
            );
            Ok(())
        }
        Err(e) => {
            spinner.finish(
                &format!("{}", styles::error(ERROR_PREFIX)),
                &format!("Compile failed ({}): ", styles::error("CE")),
            );
            Err(e.into())
        }
    }
}

fn fetch_task_data(url: &str, cookie: Option<String>) -> Result<TaskData, AppError> {
    let client = AtCoderClient::new(cookie);
    let spinner = Spinner::new("Fetching samples...");

    match client.get_task(url) {
        Ok(val) => {
            spinner.finish(
                &format!("{}", styles::success("200 OK")),
                &format!("{} sample(s) fetched.", val.samples.len()),
            );
            Ok(val)
        }
        Err(e) => {
            let code = e
                .as_http()
                .and_then(|http| http.status())
                .map_or_else(|| String::from(ERROR_PREFIX), |s| s.to_string());
            spinner.finish(
                &format!("{}", styles::error(&code)),
                "Failed to fetch samples:",
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
    info_ln!("Sample #{}", case_num);
    let mut is_ac = false;

    match runner::run_case(
        run_command,
        run_flags,
        &sample.input,
        gnu_time,
        time_limit_secs,
    ) {
        Ok(result) => {
            info_ln!("time: {}", styles::dim(&format!("{:?}", result.duration)));

            if let Some(memory_mb) = result.memory_mb {
                info_ln!("memory: {}", styles::dim(&format!("{:.3}mb", memory_mb)));
            }

            if compare_lines_trimmed(&result.output, &sample.output) {
                success_ln!("{}", styles::success("AC"));
                is_ac = true;
            } else {
                failed_ln!("{}", styles::warn("WA"));
                println!("{}:\n{}", styles::dim("Expected"), sample.output);
                println!("{}:\n{}", styles::dim("Actual"), result.output);
            }
        }
        Err(RunError::Timeout { duration }) => {
            info_ln!("time: {}", styles::dim(&format!("{:?}", duration)));
            failed_ln!("{} (Time Limit Exceeded)", styles::error("TLE"));
        }
        Err(RunError::RuntimeError { status, stderr }) => {
            failed_ln!("{} (Runtime Error)", styles::error("RE"));
            println!("{}:\n  {}", styles::dim("Exit Code"), status);
            if !stderr.is_empty() {
                println!("{}:\n  {}", styles::dim("Log"), stderr);
            }
        }
        Err(RunError::StartFailed(source)) => {
            error_ln!("System Error (Could not run the binary)");
            return Err(source.into());
        }
    }

    println!();
    Ok(is_ac)
}

fn load_config() -> Result<Option<Config>, AppError> {
    utils::load_config()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_lines_ignoring_whitespace() {
        assert!(compare_lines_trimmed("1 2\n3", "1 2\n3"));
        assert!(compare_lines_trimmed("1 2  \n\t3 \n", "1 2\n3"));
        // 行末改行の有無は等価
        assert!(compare_lines_trimmed("a\nb", "a\nb\n"));
        assert!(!compare_lines_trimmed("a\nb", "a\nc"));
        assert!(!compare_lines_trimmed("a\nb", "ab"));
    }
}
