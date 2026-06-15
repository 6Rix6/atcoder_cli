use std::collections::HashMap;

use console::{Term, style};

use crate::{
    atcoder::{self, client::AtCoderClient, models::TaskInfo},
    config::{self, error::ConfigError, models::Config},
    constants::REVEL_SESSION,
    cookie::{self, storage::CookieStorage},
    process::{compiler, runner},
    styles,
};

/// Handler for 'acc test <task_id>'
pub async fn run(task_id: &str) {
    let Some(task_info) = atcoder::utils::get_task_info(task_id) else {
        eprintln!("{}: Invalid task URL or ID.", styles::error("Error"));
        std::process::exit(1);
    };

    let config = match load_config() {
        Ok(Some(config)) => config,
        Ok(None) => {
            eprintln!("{}: No config file found.", styles::error("Error"));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("{}: {}", styles::error("Error"), e);
            std::process::exit(1);
        }
    };

    println!("{}: {}", styles::success("Running tests for task"), task_id);

    let term = Term::stdout();

    let vars = build_vars(&task_info);

    if let Some(compiler_template) = config.compiler {
        term.write_line("Compiling source...").unwrap();

        let compiler_command = expand_template(&compiler_template, &vars);
        let compiler_flags: Vec<String> = config
            .compiler_flags
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|f| expand_template(f, &vars))
            .collect();

        if let Err(e) = compiler::compile(&compiler_command, &compiler_flags).await {
            term.clear_last_lines(1).unwrap();

            eprintln!("{}", styles::error("Compile failed:"));
            eprintln!("{}", e);

            std::process::exit(1);
        }

        term.clear_last_lines(1).unwrap();
        term.write_line(&format!("{}", styles::success("Compile finished").bold()))
            .unwrap();
    }

    let cookie_path = cookie::utils::get_cookie_path();
    let storage = CookieStorage::new(cookie_path);

    let cookie = config.cookie.or_else(|| match storage.get(REVEL_SESSION) {
        Ok(Some(val)) => Some(val),
        _ => None,
    });

    let client = AtCoderClient::new(cookie);

    // TODO: show error code
    let samples = client
        .get_samples(&task_info.url)
        .await
        .expect("Failed to fetch samples");

    println!("Running {} sample(s)...\n", samples.len());

    let mut passed = 0;

    let run_command = expand_template(&config.run, &vars);
    let run_flags: Vec<String> = config
        .run_flags
        .as_ref()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|f| expand_template(f, &vars))
        .collect();

    for (i, sample) in samples.iter().enumerate() {
        let case_num = i + 1;
        let input_data = sample.input.clone();
        let expected_output = sample.output.clone();

        match runner::run_case(&run_command, &run_flags, &input_data).await {
            Ok(result) => {
                let actual_trimmed = result.output.trim();
                let expected_trimmed = expected_output.trim();

                let time_str = format!("({:?})", result.duration);

                if actual_trimmed == expected_trimmed {
                    println!(
                        "Case #{}: {} {}",
                        case_num,
                        styles::success("AC"),
                        style(time_str).dim()
                    );
                    passed += 1;
                } else {
                    println!(
                        "Case #{}: {} {}",
                        case_num,
                        styles::error("WA"),
                        style(time_str).dim()
                    );
                    println!("  {}:\n{}", style("Expected").dim(), expected_trimmed);
                    println!("  {}:\n{}", style("Actual").dim(), actual_trimmed);
                }
            }
            Err(e) => {
                println!(
                    "Case #{}: {}",
                    case_num,
                    styles::error("RE (Runtime Error)")
                );
                println!("  {}: {}", style("Details").dim(), e);
            }
        }
    }

    let result_style = if passed == samples.len() {
        style(format!("{}/{} passed", passed, samples.len()))
            .bold()
            .green()
    } else {
        style(format!("{}/{} passed", passed, samples.len()))
            .bold()
            .yellow()
    };

    println!("\nResult: {}", result_style);
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
            eprintln!(
                "Warning: unknown template variable '${{{}}}' in \"{}\"",
                var_name, template
            );
            rest = &rest[start + end + 1..];
        } else {
            break;
        }
    }

    result
}

fn build_vars(task_info: &TaskInfo) -> HashMap<&'static str, &str> {
    let mut vars: HashMap<&'static str, &str> = HashMap::new();
    vars.insert("CONTEST_ID", &task_info.contest_id);
    vars.insert("TASK_ID", &task_info.task_id);
    vars.insert("TASK_CHAR", &task_info.task_char);

    vars
}

fn load_config() -> Result<Option<Config>, ConfigError> {
    match config::utils::load_local_config() {
        Ok(result) => return Ok(result),
        Err(e) => {
            eprintln!("{}: {}", styles::warn("Warning"), e);
            println!("Continuing with global settings...");
        }
    }

    config::utils::load_global_config()
}
