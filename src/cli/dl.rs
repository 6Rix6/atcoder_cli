use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::{
    atcoder::{self, AtcoderError, client::AtCoderClient, models::TaskPage},
    cli::{
        Outcome,
        event::{self, CliEvent, CommonEvent, Task},
        styles, utils,
    },
    error::AppError,
};

#[derive(Serialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum DlEvent {
    MarkdownSaved { path: String },
    MarkdownDownloaded { text: String },
}

impl CliEvent for DlEvent {
    fn print_human(&self) {
        match self {
            DlEvent::MarkdownSaved { path } => {
                success_ln!("Saved to: {}", styles::success(&path));
            }
            DlEvent::MarkdownDownloaded { text } => println!("{}", text),
        }
    }
}

/// Download a problem statement as Markdown.
#[derive(Debug, Args)]
pub struct DlArgs {
    /// Task ID or problem URL (e.g. "abc350_a" or the tasks/... URL)
    pub task_id: String,

    /// Output file path (writes to stdout when omitted)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Replace MathJax-specific alias commands (e.g. \lt, \gt) with standard TeX
    #[arg(short, long)]
    pub replace_mathjax: bool,
}

pub fn download(args: &DlArgs) -> Result<Outcome, AppError> {
    let Some(task_info) = atcoder::utils::get_task_info(&args.task_id) else {
        return Err(AppError::InvalidTask(args.task_id.clone()));
    };

    let cookie = utils::cookie::session_cookie()?;

    let page = fetch_task_page(&task_info.url, cookie, args)?;

    match &args.output {
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, &page.problem_md)?;
            event::emit(&DlEvent::MarkdownSaved {
                path: path.display().to_string(),
            });
        }
        None => event::emit(&DlEvent::MarkdownDownloaded {
            text: page.problem_md,
        }),
    }

    Ok(Outcome::Success)
}

fn fetch_task_page(
    url: &str,
    cookie: Option<String>,
    args: &DlArgs,
) -> Result<TaskPage, AtcoderError> {
    let client = AtCoderClient::new(cookie);
    let task = Task::start(
        &CommonEvent::FetchStarted {
            url: String::from(url),
        },
        "Fetching task page...",
    );

    match client.get_page(url, args.replace_mathjax) {
        Ok(data) => {
            task.finish_ok(
                &CommonEvent::FetchFinished {
                    success: true,
                    status_code: Some(String::from("200 OK")),
                },
                Some("200 OK"),
                &format!("Fetched {}", url),
            );

            Ok(data)
        }
        Err(e) => {
            let code = e
                .as_http()
                .and_then(|http| http.status())
                .map_or_else(|| None, |s| Some(s.to_string()));

            task.finish_err(
                &CommonEvent::FetchFinished {
                    success: false,
                    status_code: code.clone(),
                },
                code.as_deref(),
                "Failed to fetch problem page.",
            );

            Err(e)
        }
    }
}
