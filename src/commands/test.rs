use std::collections::HashMap;
use std::io::Write;
use std::process::Command;

use crate::constants::REVEL_SESSION;
use crate::utils::atcoder_client::{self, AtCoderClient, TaskInfo};
use crate::utils::config;
use crate::utils::cookie_storage::{self, CookieStorage};

/// Handler for 'acc test <task_id>'
pub async fn run(task_id: &str) {
    println!("Running tests for task: {}", task_id);

    let config = match config::load_config() {
        Ok(Some(config)) => config,
        Ok(None) => {
            eprintln!("No config file found.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let Some(task_info) = atcoder_client::get_task_info(task_id) else {
        eprintln!("Invalid task Url or Id.");
        return;
    };

    let vars = build_vars(&task_info);

    if let Some(compiler_template) = config.compiler {
        println!("Compiling source...");
        let compiler = expand_template(&compiler_template, &vars);

        let flags: Vec<String> = config
            .compiler_flags
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|f| expand_template(f, &vars))
            .collect();

        let compile_status = Command::new(&compiler)
            .args(flags)
            .status()
            .expect("Compile failed");

        if !compile_status.success() {
            eprintln!("Compile failed");
            return;
        }
    }

    let cookie_path = cookie_storage::get_cookie_path();
    let storage = CookieStorage::new(cookie_path);

    let cookie = config.cookie.or_else(|| match storage.get(REVEL_SESSION) {
        Ok(Some(val)) => Some(val),
        _ => None,
    });

    let client = AtCoderClient::new(cookie);

    let samples = client
        .get_samples(&task_info.url)
        .await
        .expect("Failed to fetch samples");

    println!("Running {} sample(s)...\n", samples.len());

    let mut passed = 0;

    let run = expand_template(&config.run, &vars);
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

        let mut process = Command::new(&run)
            .args(&run_flags)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start");

        if let Some(mut stdin) = process.stdin.take() {
            stdin
                .write_all(input_data.as_bytes())
                .expect("Failed to write stdin");
        }

        let output = process.wait_with_output().expect("Failed to get output");
        let actual_output = String::from_utf8_lossy(&output.stdout);

        let actual_trimmed = actual_output.trim();
        let expected_trimmed = expected_output.trim();

        if actual_trimmed == expected_trimmed {
            println!("Case #{}: AC", case_num);
            passed += 1;
        } else {
            println!("Case #{}: WA", case_num);
            println!("  Expected:\n{}", expected_trimmed);
            println!("  Actual:\n{}", actual_trimmed);
        }
    }

    println!("\nResult: {}/{} passed", passed, samples.len());
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
            eprintln!("Warning: unknown template variable '${{{}}}' in \"{}\"", var_name, template);
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
