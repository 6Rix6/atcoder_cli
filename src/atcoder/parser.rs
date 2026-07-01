use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use crate::atcoder::{
    markdown,
    models::{SampleInput, TaskLanguage, TaskPage},
};

const SAMPLE_INPUT_KEY_JA: &str = "入力例";
const SAMPLE_OUTPUT_KEY_JA: &str = "出力例";
const SAMPLE_INPUT_KEY_EN: &str = "Sample Input";
const SAMPLE_OUTPUT_KEY_EN: &str = "Sample Output";

pub struct Parser {
    document: Html,
}

impl Parser {
    pub fn new(html: &str) -> Self {
        Self {
            document: Html::parse_document(html),
        }
    }

    /// Extract sample input/output pairs from a problem page HTML.
    pub fn samples(&self) -> Vec<SampleInput> {
        let body = self.get_task_statement(TaskLanguage::English);

        let Some(body) = body else {
            return vec![];
        };

        let part_selector = Selector::parse(".part").unwrap();
        let h3_selector = Selector::parse("h3").unwrap();
        let pre_selector = Selector::parse("pre").unwrap();

        let sample_sections: Vec<ElementRef> = body
            .select(&part_selector)
            .filter(|el| {
                let heading = el
                    .select(&h3_selector)
                    .next()
                    .map(|h| h.text().collect::<String>())
                    .unwrap_or_default();

                heading.contains(SAMPLE_INPUT_KEY_JA)
                    || heading.contains(SAMPLE_OUTPUT_KEY_JA)
                    || heading.contains(SAMPLE_INPUT_KEY_EN)
                    || heading.contains(SAMPLE_OUTPUT_KEY_EN)
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

    pub fn page(&self) -> Option<TaskPage> {
        let container = self.get_main_container()?;
        let body = self.get_task_statement(TaskLanguage::Japanese)?;

        let title_selector = Selector::parse("span.h2").unwrap();
        let title = container
            .select(&title_selector)
            .next()
            .map(|elm| elm.text().next().unwrap_or_default().trim())
            .unwrap_or_default();

        let constraints_selector = Selector::parse("p").unwrap();
        let constraints = container
            .select(&constraints_selector)
            .next()
            .map(|elm| elm.text().next().unwrap_or_default().trim())
            .unwrap_or_default();

        let (sec, mem) = Self::extract_limits(constraints).unwrap_or((2, 1024));

        let md = format!(
            "## {}\n{}\n\n{}",
            &title,
            &constraints,
            markdown::html_to_md(&body.html())
        );

        Some(TaskPage {
            title: title.to_string(),
            time_limit: sec,
            memory_limit: mem,
            problem_md: md,
        })
    }

    fn get_task_statement(&self, lang: TaskLanguage) -> Option<ElementRef<'_>> {
        let container = self.get_main_container()?;

        Selector::parse(&format!("span.lang-{}", lang.code()))
            .ok()
            .and_then(|s| container.select(&s).next())
            .or_else(|| {
                Selector::parse("div#task-statement")
                    .ok()
                    .and_then(|s| container.select(&s).next())
            })
    }

    fn get_main_container(&self) -> Option<ElementRef<'_>> {
        Selector::parse("#main-container")
            .ok()
            .and_then(|s| self.document.select(&s).next())
    }

    fn extract_limits(text: &str) -> Option<(u32, u32)> {
        let re = Regex::new(
        r"(?:実行時間制限|Time Limit):\s*(\d+)\s*sec.*?(?:メモリ制限|Memory Limit):\s*(\d+)\s*MiB",
    )
    .unwrap();

        if let Some(caps) = re.captures(text) {
            let time_limit: u32 = caps[1].parse().ok()?;
            let memory_limit: u32 = caps[2].parse().ok()?;

            Some((time_limit, memory_limit))
        } else {
            None
        }
    }
}
