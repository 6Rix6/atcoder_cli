use std::time::Duration;

use reqwest::{Client, Response, redirect::Policy};

use crate::{
    atcoder::{
        error::AtcoderError,
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
            .expect("failed to build HTTP client");

        Self { client, cookie }
    }

    /// Fetch a page from AtCoder using the inner client and cookie.
    pub async fn fetch_page(&self, url: &str) -> Result<Response, AtcoderError> {
        let mut request = self.client.get(url);

        if let Some(ref cookie_value) = self.cookie {
            // Skip malformed values instead of panicking inside reqwest.
            if is_valid_session_value(cookie_value) {
                request = request.header("Cookie", format!("{}={};", REVEL_SESSION, cookie_value));
            }
        }

        request.send().await.map_err(AtcoderError::from)
    }

    /// Fetch HTML content from the given URL.
    pub async fn fetch_html(&self, url: &str) -> Result<String, AtcoderError> {
        let response = self.fetch_page(url).await?;
        let response = response.error_for_status()?;

        response.text().await.map_err(AtcoderError::from)
    }

    /// Fetch a problem page and extract both its metadata and sample cases.
    pub async fn get_task(&self, url: &str) -> Result<TaskData, AtcoderError> {
        let html = self.fetch_html(url).await?;
        let parser = Parser::new(&html);
        let page = parser.page().ok_or_else(|| AtcoderError::Parse {
            url: url.to_string(),
        })?;

        Ok(TaskData {
            page,
            samples: parser.samples(),
        })
    }

    /// Fetch a problem page and extract its metadata.
    pub async fn get_page(&self, url: &str) -> Result<TaskPage, AtcoderError> {
        let html = self.fetch_html(url).await?;

        Parser::new(&html)
            .page()
            .ok_or_else(|| AtcoderError::Parse {
                url: url.to_string(),
            })
    }
}
