use clap::Args;

use crate::{
    atcoder::{self, client::AtCoderClient},
    cli::{Outcome, utils},
    error::AppError,
};

/// Download a problem statement as Markdown.
#[derive(Debug, Args)]
pub struct DlArgs {
    /// Task ID or problem URL (e.g. "abc350_a" or the tasks/... URL)
    pub task_id: String,
}

pub fn download(args: &DlArgs) -> Result<Outcome, AppError> {
    let Some(task_info) = atcoder::utils::get_task_info(&args.task_id) else {
        return Err(AppError::InvalidTask(args.task_id.clone()));
    };

    let cookie = utils::session_cookie()?;

    let client = AtCoderClient::new(cookie);
    let page = client.get_page(&task_info.url)?;

    println!("{}", page.problem_md);

    Ok(Outcome::Success)
}
