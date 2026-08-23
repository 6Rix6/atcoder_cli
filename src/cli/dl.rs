use std::path::PathBuf;

use clap::Args;

use crate::{
    atcoder::{self, client::AtCoderClient},
    cli::{Outcome, styles, utils},
    error::AppError,
};

/// Download a problem statement as Markdown.
#[derive(Debug, Args)]
pub struct DlArgs {
    /// Task ID or problem URL (e.g. "abc350_a" or the tasks/... URL)
    pub task_id: String,

    /// Output file path (writes to stdout when omitted)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

pub fn download(args: &DlArgs) -> Result<Outcome, AppError> {
    let Some(task_info) = atcoder::utils::get_task_info(&args.task_id) else {
        return Err(AppError::InvalidTask(args.task_id.clone()));
    };

    let cookie = utils::session_cookie()?;

    let client = AtCoderClient::new(cookie);
    let page = client.get_page(&task_info.url)?;

    match &args.output {
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, &page.problem_md)?;
            success_ln!("Saved to: {}", styles::success(&path.display().to_string()));
        }
        None => println!("{}", page.problem_md),
    }

    Ok(Outcome::Success)
}
