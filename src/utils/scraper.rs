use reqwest::{Client, Response, redirect::Policy};
use scraper::{ElementRef, Html, Selector};

#[derive(Debug, Clone)]
pub struct SampleInput {
    pub input: String,
    pub output: String,
}

pub async fn fetch_page(
    url: &str,
    cookie: Option<&str>,
) -> Result<Response, reqwest::Error> {
    let client = Client::builder().redirect(Policy::none()).build()?;

    let mut request = client.get(url);

    if let Some(cookie_value) = cookie {
        request = request.header("Cookie", format!("REVEL_SESSION={};", cookie_value));
    }

    let response = request.send().await?;
    Ok(response)
}

/// Fetch HTML from the given URL, optionally with a session cookie.
async fn fetch_html(url: &str, cookie: Option<&str>) -> Result<String, reqwest::Error> {
    let response = fetch_page(url, cookie).await?;
    let html = response.text().await?;

    Ok(html)
}

/// Extract sample input/output pairs from a problem page HTML.
fn extract_samples(html: &str) -> Vec<SampleInput> {
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

    // Collect all .part elements whose h3 heading is a sample input or output
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

    // Pair up: even index = input, odd index = output
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

/// Generate a task URL from a task ID or full URL.
pub fn generate_task_url(id_or_url: &str) -> Option<String> {
    // Extract the last path-like segment (e.g. from full URL or raw ID)
    let task_id = id_or_url
        .split('/')
        .filter(|s| !s.is_empty())
        .last()?
        .split('?')
        .next()?;

    // Must match: lowercase alphanumeric segments joined by underscores
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

    // Everything except the last segment is the contest ID
    let contest_id = parts[..parts.len() - 1].join("-");

    Some(format!(
        "https://atcoder.jp/contests/{}/tasks/{}",
        contest_id, task_id
    ))
}

/// Fetch and return all sample input/output pairs for the given problem URL.
pub async fn get_samples(
    url: &str,
    cookie: Option<&str>,
) -> Result<Vec<SampleInput>, Box<dyn std::error::Error>> {
    let html = fetch_html(url, cookie).await?;
    let samples = extract_samples(&html);
    Ok(samples)
}
