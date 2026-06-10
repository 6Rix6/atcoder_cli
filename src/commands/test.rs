use serde::Deserialize;
use std::io::Write;
use std::process::Command;

use crate::constants::REVEL_SESSION;
use crate::utils::atcoder_client::{self, AtCoderClient};
use crate::utils::cookie_storage::{self, CookieStorage};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub compiler: String,
    pub flags: Vec<String>,
    pub run: String,
    pub cookie: Option<String>,
}

/// Handler for 'acc test <task_id>'
pub async fn run(task_id: &str) {
    println!("Running tests for task: {}", task_id);

    // TODO: add error handling
    let toml_str = std::fs::read_to_string("atcoder.toml").unwrap();

    let config: Config = toml::from_str(&toml_str).expect("Failed to parse atcoder.toml");

    let final_compiler = config.compiler;
    let final_flags = config.flags;
    let final_run = config.run;

    println!("Compiling source...");
    let compile_status = Command::new(&final_compiler)
        .args(&final_flags)
        .status()
        .expect("Compile failed");

    if !compile_status.success() {
        eprintln!("Compile failed");
        return;
    }

    let cookie_path = cookie_storage::get_cookie_path();
    let storage = CookieStorage::new(cookie_path);

    let cookie = config
        .cookie
        .or_else(|| match storage.get(REVEL_SESSION) {
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

    for (i, sample) in samples.iter().enumerate() {
        let case_num = i + 1;
        let input_data = sample.input.clone();
        let expected_output = sample.output.clone();

        let mut process = Command::new(&final_run)
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
