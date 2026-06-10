use std::io::Write;
use std::process::Command;

use crate::constants::REVEL_SESSION;
use crate::utils::atcoder_client::{self, AtCoderClient};
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

    if let Some(compiler) = config.compiler {
        println!("Compiling source...");

        let empty_flags = Vec::new();
        let flags = config.compiler_flags.as_ref().unwrap_or(&empty_flags);

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

    let Some(task_url) = atcoder_client::generate_task_url(task_id) else {
        eprintln!("Invalid task Url or Id.");
        return;
    };

    let client = AtCoderClient::new(cookie);

    let samples = client
        .get_samples(&task_url)
        .await
        .expect("Failed to fetch samples");

    println!("Running {} sample(s)...\n", samples.len());

    let mut passed = 0;

    let empty_flags = Vec::new();
    let run_flags = config.run_flags.as_ref().unwrap_or(&empty_flags);

    for (i, sample) in samples.iter().enumerate() {
        let case_num = i + 1;
        let input_data = sample.input.clone();
        let expected_output = sample.output.clone();

        let mut process = Command::new(&config.run)
            .args(run_flags)
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
