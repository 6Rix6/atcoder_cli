use std::collections::HashMap;

use clap::Args;

use crate::{
    atcoder::{self, client::AtCoderClient, models::SampleInput},
    cli::{
        styles::{self, ERROR_PREFIX, SUCCESS_PREFIX, spinner::Spinner},
        utils,
    },
    config::{self, error::ConfigError, models::Config},
    process::{
        compiler,
        runner::{self, RunError},
    },
    utils::gnu_time::get_gnu_time_path,
};

#[derive(Debug, Args)]
pub struct TestArgs {
    pub task_id: String,
}

/// Handler for 'acc test <task_id>'
pub async fn run(args: &TestArgs) {
    let Some(task_info) = atcoder::utils::get_task_info(&args.task_id) else {
        error_ln!("Invalid task URL or ID.");
        std::process::exit(1);
    };

    success_ln!("Running for task: {}", styles::success(&task_info.task_id));

    let config = match load_config() {
        Ok(Some(config)) => config,
        Ok(None) => {
            error_ln!("No config file found.");
            std::process::exit(1);
        }
        Err(e) => {
            error_ln!("{}", e);
            std::process::exit(1);
        }
    };

    let vars = task_info.to_map();

    if let Some(compiler_template) = &config.compiler {
        let flags = config.compiler_flags.as_deref().unwrap_or_default();
        if compile(compiler_template, flags, &vars).await.is_err() {
            std::process::exit(1);
        }
    }

    let cookie = utils::get_revel_session();

    let Some(samples) = fetch_samples(&task_info.url, cookie).await else {
        std::process::exit(1);
    };

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

    for (i, sample) in samples.iter().enumerate() {
        if execute_test_case(
            i + 1,
            &run_command,
            &run_flags,
            sample,
            gnu_time_path.as_ref().and_then(|p| p.to_str()),
        )
        .await
        {
            passed += 1;
        }
    }

    if passed == samples.len() {
        success_ln!("Test {}", styles::success("succeeded"));
        info_ln!("Submit here: {}", &task_info.submit_url);
    } else {
        failed_ln!(
            "Test {}: {}/{} passed",
            styles::error("failed"),
            passed,
            samples.len()
        );
    }
}

fn compare_lines_trimmed(s1: &str, s2: &str) -> bool {
    s1.lines()
        .map(|l| l.trim())
        .eq(s2.lines().map(|l| l.trim()))
}

async fn compile(
    compiler_template: &str,
    compiler_flags: &[String],
    vars: &HashMap<&str, &str>,
) -> Result<(), ()> {
    let spinner = Spinner::new("Compiling source...");

    let compiler_command = expand_template(compiler_template, vars);
    let compiler_flags: Vec<String> = compiler_flags
        .iter()
        .map(|f| expand_template(f, vars))
        .collect();

    if let Err(e) = compiler::compile(&compiler_command, &compiler_flags).await {
        spinner.finish(
            &format!("{}", styles::error(ERROR_PREFIX)),
            &format!("Compile failed ({}): ", styles::error("CE")),
        );
        eprintln!("{}", e);
        return Err(());
    }

    spinner.finish(
        &format!("{}", styles::success(SUCCESS_PREFIX)),
        "Compile finished.",
    );
    Ok(())
}

async fn fetch_samples(url: &str, cookie: Option<String>) -> Option<Vec<SampleInput>> {
    let client = AtCoderClient::new(cookie);
    let spinner = Spinner::new("Fetching samples...");

    match client.get_samples(url).await {
        Ok(val) => {
            spinner.finish(
                &format!("{}", styles::success("200 OK")),
                &format!("{} sample(s) fetched.", val.len()),
            );
            Some(val)
        }
        Err(e) => {
            let code = e
                .status()
                .map_or_else(|| String::from(ERROR_PREFIX), |s| s.to_string());
            spinner.finish(
                &format!("{}", styles::error(&code)),
                "Failed to fetch samples:",
            );
            eprintln!("{}", e);
            None
        }
    }
}

async fn execute_test_case(
    case_num: usize,
    run_command: &str,
    run_flags: &[String],
    sample: &SampleInput,
    gnu_time: Option<&str>,
) -> bool {
    info_ln!("Sample #{}", case_num);
    let mut is_ac = false;

    match runner::run_case(run_command, run_flags, &sample.input, gnu_time).await {
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
        Err(RunError::StartFailed(e)) => {
            error_ln!("System Error (Could not run the binary)");
            println!("{}:\n  {}", styles::dim("Details"), e);
        }
    }

    println!();
    is_ac
}

fn expand_template(template: &str, vars: &HashMap<&str, &str>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("${{{}}}", key), value);
    }

    let mut rest = result.as_str();
    while let Some(start) = rest.find("${") {
        if let Some(end) = rest[start..].find('}') {
            let var_name = &rest[start + 2..start + end];
            warn_ln!(
                "unknown template variable '${{{}}}' in \"{}\"",
                var_name,
                template
            );
            rest = &rest[start + end + 1..];
        } else {
            break;
        }
    }

    result
}

fn load_config() -> Result<Option<Config>, ConfigError> {
    match config::utils::load_local_config() {
        Ok(result) => Ok(result),
        Err(e) => {
            warn_ln!("{}", e);
            warn_ln!("Continuing with global settings...");
            config::utils::load_global_config()
        }
    }
}
