use std::{collections::HashMap, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    atcoder::{self, client::AtCoderClient, models::SampleInput},
    cli::styles,
    config::{self, error::ConfigError, models::Config},
    constants::{CONFIG_FILE_NAME, REVEL_SESSION},
    cookie::{self, storage::CookieStorage},
    process::{
        compiler,
        runner::{self, RunError},
    },
    utils::gnu_time::get_gnu_time_path,
};

/// Handler for 'acc test <task_id>'
pub async fn run(task_id: &str) {
    let Some(task_info) = atcoder::utils::get_task_info(task_id) else {
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

    let cookie = get_cookie(config.cookie.as_deref());

    let Some(samples) = fetch_samples(&task_info.url, cookie).await else {
        std::process::exit(1);
    };

    print!("\n");

    let gnu_time_path = get_gnu_time_path();
    if gnu_time_path.is_none() {
        warn_ln!("GNU time is not available. Memory usage will not be measured.");
    }

    print!("\n");

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
    let pb = create_spinner("Compiling source...");

    let compiler_command = expand_template(compiler_template, vars);
    let compiler_flags: Vec<String> = compiler_flags
        .iter()
        .map(|f| expand_template(f, vars))
        .collect();

    if let Err(e) = compiler::compile(&compiler_command, &compiler_flags).await {
        finish_spinner(
            &pb,
            &format!("{}", styles::error("ERROR")),
            &format!("Compile failed ({}): ", styles::error("CE")),
        );
        eprintln!("{}", e);
        return Err(());
    }

    finish_spinner(
        &pb,
        &format!("{}", styles::success("✓")),
        "Compile finished.",
    );
    Ok(())
}

fn get_cookie(config_cookie: Option<&str>) -> Option<String> {
    let cookie_path = cookie::utils::get_cookie_path();
    let storage = CookieStorage::new(&cookie_path);

    let is_from_config = config_cookie.is_some();
    let cookie = config_cookie
        .map(|s| s.to_string())
        .or_else(|| storage.get(REVEL_SESSION).ok().flatten());

    match &cookie {
        Some(_) => {
            let path_str = if is_from_config {
                CONFIG_FILE_NAME.to_string()
            } else {
                cookie_path.display().to_string()
            };
            success_ln!("Using Cookie in {}", path_str);
        }
        None => {
            warn_ln!("Cookie is not set.");
        }
    };

    cookie
}

async fn fetch_samples(url: &str, cookie: Option<String>) -> Option<Vec<SampleInput>> {
    let client = AtCoderClient::new(cookie);
    let pb = create_spinner("Fetching samples...");

    match client.get_samples(url).await {
        Ok(val) => {
            finish_spinner(
                &pb,
                &format!("{}", styles::success("200 OK")),
                &format!("{} sample(s) fetched.", val.len()),
            );
            Some(val)
        }
        Err(e) => {
            let code = e
                .status()
                .map_or_else(|| "ERROR".to_string(), |s| s.to_string());
            finish_spinner(
                &pb,
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

    print!("\n");
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

fn create_spinner(msg: &'static str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template(&format_ln!("{spinner:}", "{{msg}}"))
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Helper to wrap up progress spinner with custom status and message
fn finish_spinner(pb: &ProgressBar, prefix: &str, msg: &str) {
    pb.set_style(ProgressStyle::with_template(&format_ln!(prefix, "{{msg}}")).unwrap());
    pb.finish_with_message(msg.to_string());
}
