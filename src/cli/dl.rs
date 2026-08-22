use clap::Args;

use crate::{
    atcoder::{self, client::AtCoderClient},
    cli::utils,
};

#[derive(Debug, Args)]
pub struct DlArgs {
    pub task_id: String,
}

pub async fn download(args: &DlArgs) {
    let Some(task_info) = atcoder::utils::get_task_info(&args.task_id) else {
        error_ln!("Invalid task URL or ID.");
        std::process::exit(1);
    };

    let cookie = utils::get_revel_session();

    let client = AtCoderClient::new(cookie);
    let page = client.get_page(&task_info.url).await;

    match page {
        Ok(Some(val)) => {
            println!("{}", val.problem_md);
        }
        Ok(None) => {
            failed_ln!("Failed to parse page.");
            std::process::exit(1);
        }
        Err(e) => {
            error_ln!("{}", e);
            std::process::exit(1);
        }
    }
}
