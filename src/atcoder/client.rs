use std::time::Duration;

use reqwest::{Client, Response, redirect::Policy};

use crate::{
    atcoder::{
        models::{TaskData, TaskPage},
        parser::Parser,
    },
    constants::{ATCODER_HTTP_TIMEOUT_SECS, REVEL_SESSION},
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
            .unwrap_or_default();

        Self { client, cookie }
    }

    /// Fetch a page from AtCoder using the inner client and cookie.
    pub async fn fetch_page(&self, url: &str) -> Result<Response, reqwest::Error> {
        let mut request = self.client.get(url);

        if let Some(ref cookie_value) = self.cookie {
            // Skip malformed values instead of panicking inside reqwest.
            if is_valid_session_value(cookie_value) {
                request = request.header("Cookie", format!("{}={};", REVEL_SESSION, cookie_value));
            }
        }

        request.send().await
    }

    /// Fetch HTML content from the given URL.
    pub async fn fetch_html(&self, url: &str) -> Result<String, reqwest::Error> {
        let response = self.fetch_page(url).await?;
        let response = response.error_for_status()?;

        response.text().await
    }

    /// Fetch a problem page and extract both its metadata and sample cases.
    pub async fn get_task(&self, url: &str) -> Result<TaskData, reqwest::Error> {
        let html = self.fetch_html(url).await?;
        let parser = Parser::new(&html);

        Ok(TaskData {
            page: parser.page(),
            samples: parser.samples(),
        })
    }

    pub async fn get_page(&self, url: &str) -> Result<Option<TaskPage>, reqwest::Error> {
        let html = self.fetch_html(url).await?;
        let parser = Parser::new(&html);
        let page = parser.page();

        Ok(page)
    }
}
