use reqwest::{Client, Response, redirect::Policy};
use scraper::{ElementRef, Html, Selector};

use crate::constants::REVEL_SESSION;

#[derive(Debug, Clone)]
pub struct SampleInput {
    pub input: String,
    pub output: String,
}

/// A client dedicated to interacting with AtCoder.
pub struct AtCoderClient {
    client: Client,
    cookie: Option<String>,
}

impl AtCoderClient {
    /// Create a new instance of the client.
    pub fn new(cookie: Option<String>) -> Self {
        // Initialize the client once to leverage connection pooling
        let client = Client::builder()
            .redirect(Policy::none())
            .build()
            .unwrap_or_default();

        Self { client, cookie }
    }

    /// Fetch a page from AtCoder using the inner client and cookie.
    pub async fn fetch_page(&self, url: &str) -> Result<Response, reqwest::Error> {
        let mut request = self.client.get(url);

        if let Some(ref cookie_value) = self.cookie {
            request = request.header("Cookie", format!("{}={};", REVEL_SESSION, cookie_value));
        }

        request.send().await
    }

    /// Fetch HTML content from the given URL.
    async fn fetch_html(&self, url: &str) -> Result<String, reqwest::Error> {
        let response = self.fetch_page(url).await?;
        response.text().await
    }

    /// Fetch and return all sample input/output pairs for the given problem.
    pub async fn get_samples(
        &self,
        url: &str,
    ) -> Result<Vec<SampleInput>, Box<dyn std::error::Error>> {
        let html = self.fetch_html(url).await?;
        let samples = Self::extract_samples(&html);
        Ok(samples)
    }

    /// Extract sample input/output pairs from a problem page HTML.
    /// (Kept as a static associated function since it only depends on the input string)
    pub fn extract_samples(html: &str) -> Vec<SampleInput> {
        let document = Html::parse_document(html);

        let part_selector = Selector::parse(".part").unwrap();
        let h3_selector = Selector::parse("h3").unwrap();
        let pre_selector = Selector::parse("pre").unwrap();

        let body = Selector::parse("span.lang-ja")
            .ok()
            .and_then(|s| document.select(&s).next())
            .or_else(|| {
                Selector::parse("span.lang-en")
                    .ok()
                    .and_then(|s| document.select(&s).next())
            })
            .or_else(|| {
                Selector::parse("div#task-statement")
                    .ok()
                    .and_then(|s| document.select(&s).next())
            });

        let Some(body) = body else {
            return vec![];
        };

        let sample_sections: Vec<ElementRef> = body
            .select(&part_selector)
            .filter(|el| {
                let heading = el
                    .select(&h3_selector)
                    .next()
                    .map(|h| h.text().collect::<String>())
                    .unwrap_or_default();

                heading.contains("入力例")
                    || heading.contains("出力例")
                    || heading.contains("Sample Input")
                    || heading.contains("Sample Output")
            })
            .collect();

        let mut samples = Vec::new();
        let mut i = 0;
        while i + 1 < sample_sections.len() {
            let input_section = &sample_sections[i];
            let output_section = &sample_sections[i + 1];

            let input = input_section
                .select(&pre_selector)
                .next()
                .map(|pre| pre.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let output = output_section
                .select(&pre_selector)
                .next()
                .map(|pre| pre.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            samples.push(SampleInput { input, output });
            i += 2;
        }

        samples
    }
}

/// Generate a task URL from a task ID or full URL.
pub fn generate_task_url(id_or_url: &str) -> Option<String> {
    let task_id = id_or_url
        .split('/')
        .filter(|s| !s.is_empty())
        .last()?
        .split('?')
        .next()?;

    let valid = task_id
        .split('_')
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric()));

    if !valid {
        return None;
    }

    let parts: Vec<&str> = task_id.split('_').collect();
    if parts.len() < 2 {
        return None;
    }

    let contest_id = parts[..parts.len() - 1].join("-");

    Some(format!(
        "https://atcoder.jp/contests/{}/tasks/{}",
        contest_id, task_id
    ))
}
