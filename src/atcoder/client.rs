use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::redirect::Policy;

use crate::{
    atcoder::{
        error::AtcoderError,
        models::{TaskData, TaskInfo, TaskPage},
        parser::Parser,
        utils::get_task_info,
    },
    constants::{ATCODER_BASE_URL, ATCODER_HTTP_TIMEOUT_SECS, REVEL_SESSION},
    cookie::utils::is_valid_session_value,
};

/// A client dedicated to interacting with AtCoder.
pub struct AtCoderClient {
    client: Client,
    cookie: Option<String>,
}

impl AtCoderClient {
    /// Create a new instance of the client.
    pub fn new(cookie: Option<String>) -> Self {
        let client = Client::builder()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(ATCODER_HTTP_TIMEOUT_SECS))
            .build()
            .expect("failed to build HTTP client");

        Self { client, cookie }
    }

    /// Fetch a page from AtCoder using the inner client and cookie.
    pub fn fetch_page(&self, url: &str) -> Result<Response, AtcoderError> {
        let mut request = self.client.get(url);

        if let Some(ref cookie_value) = self.cookie {
            // Skip malformed values instead of panicking inside reqwest.
            if is_valid_session_value(cookie_value) {
                request = request.header("Cookie", format!("{}={};", REVEL_SESSION, cookie_value));
            }
        }

        request.send().map_err(AtcoderError::from)
    }

    /// Fetch HTML content from the given URL.
    pub fn fetch_html(&self, url: &str) -> Result<String, AtcoderError> {
        let response = self.fetch_page(url)?;
        let response = response.error_for_status()?;

        response.text().map_err(AtcoderError::from)
    }

    /// Fetch a problem page and extract both its metadata and sample cases.
    pub fn get_task(&self, url: &str) -> Result<TaskData, AtcoderError> {
        let html = self.fetch_html(url)?;
        let parser = Parser::new(&html);
        let page = parser.page(false).ok_or_else(|| AtcoderError::Parse {
            url: url.to_string(),
        })?;

        Ok(TaskData {
            page,
            samples: parser.samples(),
        })
    }

    /// Fetch a problem page and extract its metadata.
    pub fn get_page(&self, url: &str, replace_mathjax: bool) -> Result<TaskPage, AtcoderError> {
        let html = self.fetch_html(url)?;

        Parser::new(&html)
            .page(replace_mathjax)
            .ok_or_else(|| AtcoderError::Parse {
                url: url.to_string(),
            })
    }

    /// Fetch the task list of a contest.
    ///
    /// The task-list page is tried first. Before a contest starts that page is
    /// unavailable, so this falls back to the point-value table on the contest
    /// top page.
    pub fn get_task_list(&self, contest_id: &str) -> Result<Vec<TaskInfo>, AtcoderError> {
        let tasks_url = format!("{}/contests/{}/tasks", ATCODER_BASE_URL, contest_id);

        if let Ok(html) = self.fetch_html(&tasks_url) {
            let infos: Vec<TaskInfo> = Parser::new(&html)
                .task_screen_names(contest_id)
                .into_iter()
                .filter_map(|name| get_task_info(&name))
                .collect();

            if !infos.is_empty() {
                return Ok(infos);
            }
        }

        let top_url = format!("{}/contests/{}", ATCODER_BASE_URL, contest_id);
        let html = self.fetch_html(&top_url)?;

        let infos: Vec<TaskInfo> = Parser::new(&html)
            .contest_top_task_chars()
            .iter()
            .map(|c| format!("{}_{}", contest_id, c))
            .filter_map(|id| get_task_info(&id))
            .collect();

        if infos.is_empty() {
            return Err(AtcoderError::NoTasksFound { url: tasks_url });
        }

        Ok(infos)
    }
}
