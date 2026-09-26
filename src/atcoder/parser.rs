use std::collections::HashSet;
use std::sync::LazyLock;

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

const POINT_HEADING_JA: &str = "配点";
const POINT_HEADING_EN: &str = "Point Value";

static PART_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".part").expect("static selector must be valid"));
static H3_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h3").expect("static selector must be valid"));
static PRE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("pre").expect("static selector must be valid"));
static TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("span.h2").expect("static selector must be valid"));
static CONSTRAINTS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("p").expect("static selector must be valid"));
static MAIN_CONTAINER_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("#main-container").expect("static selector must be valid"));
static TASK_STATEMENT_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div#task-statement").expect("static selector must be valid"));
static LIMITS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:実行時間制限|Time Limit):\s*(\d+)\s*sec.*?(?:メモリ制限|Memory Limit):\s*(\d+)\s*MiB",
    )
    .expect("static regex must be valid")
});
static TASK_LINK_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a[href]").expect("static selector must be valid"));
static STATEMENT_CONTAINER_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("#contest-statement").expect("static selector must be valid"));
static TABLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("table").expect("static selector must be valid"));
static ROW_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("tbody tr").expect("static selector must be valid"));
static CELL_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("td").expect("static selector must be valid"));

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
        let Some(body) = self.get_task_statement(TaskLanguage::English) else {
            return vec![];
        };

        let sample_sections: Vec<ElementRef> = body
            .select(&PART_SELECTOR)
            .filter(|el| {
                let heading = el
                    .select(&H3_SELECTOR)
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
                .select(&PRE_SELECTOR)
                .next()
                .map(|pre| pre.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let output = output_section
                .select(&PRE_SELECTOR)
                .next()
                .map(|pre| pre.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            samples.push(SampleInput { input, output });
            i += 2;
        }

        samples
    }

    pub fn page(&self, replace_mathjax: bool) -> Option<TaskPage> {
        let container = self.get_main_container()?;
        let body = self.get_task_statement(TaskLanguage::Japanese)?;

        let title = container
            .select(&TITLE_SELECTOR)
            .next()
            .map(|elm| elm.text().next().unwrap_or_default().trim())
            .unwrap_or_default();

        let constraints = container
            .select(&CONSTRAINTS_SELECTOR)
            .next()
            .map(|elm| elm.text().next().unwrap_or_default().trim())
            .unwrap_or_default();

        let (sec, mem) = Self::extract_limits(constraints).unwrap_or((2, 1024));

        let md = format!(
            "## {}\n{}\n\n{}",
            title,
            constraints,
            markdown::html_to_md(&body.html(),replace_mathjax)
        );

        Some(TaskPage {
            title: title.to_string(),
            time_limit: sec,
            memory_limit: mem,
            problem_md: md,
        })
    }

    /// Extract task screen names linked from a contest task-list page
    /// (`/contests/{id}/tasks`).
    ///
    /// Each task is linked twice (index cell and title cell); duplicates are
    /// removed while preserving first-seen order.
    pub fn task_screen_names(&self, contest_id: &str) -> Vec<String> {
        let prefix = format!("/contests/{}/tasks/", contest_id);

        let mut seen = HashSet::new();
        let mut names = Vec::new();

        for anchor in self.document.select(&TASK_LINK_SELECTOR) {
            let href = anchor.value().attr("href").unwrap_or_default();
            if !href.starts_with(&prefix) {
                continue;
            }

            let name = href.rsplit('/').next().unwrap_or_default();
            if name.is_empty() || !seen.insert(name.to_string()) {
                continue;
            }
            names.push(name.to_string());
        }

        names
    }

    /// Extract problem characters ("a", "b", ...) from the point-value table
    /// of a contest top page, which is the only source available before a
    /// contest starts (`配点` / `Point Values` headings).
    pub fn contest_top_task_chars(&self) -> Vec<String> {
        let Some(container) = self.document.select(&STATEMENT_CONTAINER_SELECTOR).next() else {
            return vec![];
        };

        for heading in container.select(&H3_SELECTOR) {
            let text = heading.text().collect::<String>();
            let trimmed = text.trim();
            if !(trimmed.starts_with(POINT_HEADING_JA) || trimmed.starts_with(POINT_HEADING_EN)) {
                continue;
            }

            let chars = Self::task_chars_from_point_table(&heading);
            if !chars.is_empty() {
                return chars;
            }
        }

        vec![]
    }

    /// Collects the problem letters from the first table following `heading`.
    fn task_chars_from_point_table(heading: &ElementRef<'_>) -> Vec<String> {
        let Some(table) = Self::first_table_after(heading) else {
            return vec![];
        };

        let mut seen = HashSet::new();
        let mut chars = Vec::new();

        for row in table.select(&ROW_SELECTOR) {
            let Some(cell) = row.select(&CELL_SELECTOR).next() else {
                continue;
            };

            let letter = cell.text().collect::<String>().trim().to_lowercase();
            if !letter.is_empty() && seen.insert(letter.clone()) {
                chars.push(letter);
            }
        }

        chars
    }

    /// Walks forward from `start` among its siblings and returns the first
    /// table element (searching descendants as well).
    fn first_table_after<'a>(start: &ElementRef<'a>) -> Option<ElementRef<'a>> {
        let mut node = start.next_sibling();
        while let Some(current) = node {
            if let Some(element) = ElementRef::wrap(current) {
                if element.value().name() == "table" {
                    return Some(element);
                }
                if let Some(table) = element.select(&TABLE_SELECTOR).next() {
                    return Some(table);
                }
            }
            node = current.next_sibling();
        }

        None
    }

    fn get_task_statement(&self, lang: TaskLanguage) -> Option<ElementRef<'_>> {
        let container = self.get_main_container()?;

        Selector::parse(&format!("span.lang-{}", lang.code()))
            .ok()
            .and_then(|s| container.select(&s).next())
            .or_else(|| container.select(&TASK_STATEMENT_SELECTOR).next())
    }

    fn get_main_container(&self) -> Option<ElementRef<'_>> {
        self.document.select(&MAIN_CONTAINER_SELECTOR).next()
    }

    fn extract_limits(text: &str) -> Option<(u32, u32)> {
        let caps = LIMITS_REGEX.captures(text)?;

        let time_limit: u32 = caps[1].parse().ok()?;
        let memory_limit: u32 = caps[2].parse().ok()?;

        Some((time_limit, memory_limit))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const JA_PROBLEM_HTML: &str = r#"
<html><body>
<div id="main-container">
  <span class="h2">A - Past ABCs</span>
  <div id="task-statement">
    <span class="lang-ja">
      <p>実行時間制限: 2 sec / メモリ制限: 1024 MiB</p>
      <div class="part"><section><h3>問題文</h3><p>長さ 6 の文字列 S が与えられます。</p></section></div>
      <div class="part"><section><h3>入力例 <var>1</var></h3><pre>ABC349</pre></section></div>
      <div class="part"><section><h3>出力例 <var>1</var></h3><pre>ABC350</pre></section></div>
      <div class="part"><section><h3>入力例 <var>2</var></h3><pre>ABC315</pre></section></div>
      <div class="part"><section><h3>出力例 <var>2</var></h3><pre>ABC316</pre></section></div>
    </span>
  </div>
</div>
</body></html>
"#;

    #[test]
    fn extracts_samples_from_japanese_page() {
        let parser = Parser::new(JA_PROBLEM_HTML);
        let samples = parser.samples();

        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].input, "ABC349");
        assert_eq!(samples[0].output, "ABC350");
        assert_eq!(samples[1].input, "ABC315");
        assert_eq!(samples[1].output, "ABC316");
    }

    #[test]
    fn extracts_samples_from_english_page() {
        let html = r#"
<html><body>
<div id="main-container">
  <span class="h2">A - Problem</span>
  <div id="task-statement">
    <span class="lang-en">
      <div class="part"><section><h3>Sample Input <var>1</var></h3><pre>1 2</pre></section></div>
      <div class="part"><section><h3>Sample Output <var>1</var></h3><pre>3</pre></section></div>
    </span>
  </div>
</div>
</body></html>
"#;

        let parser = Parser::new(html);
        let samples = parser.samples();

        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].input, "1 2");
        assert_eq!(samples[0].output, "3");
    }

    #[test]
    fn ignores_unrelated_parts_and_dangling_input() {
        let html = r#"
<html><body>
<div id="main-container">
  <div id="task-statement">
    <span class="lang-ja">
      <div class="part"><section><h3>問題文</h3><p>本文。</p></section></div>
      <div class="part"><section><h3>入力</h3><p>入力の説明。</p></section></div>
      <div class="part"><section><h3>出力</h3><p>出力の説明。</p></section></div>
      <div class="part"><section><h3>入力例 <var>1</var></h3><pre>x</pre></section></div>
    </span>
  </div>
</div>
</body></html>
"#;

        let parser = Parser::new(html);
        assert!(parser.samples().is_empty());
    }

    #[test]
    fn parses_page_metadata() {
        let parser = Parser::new(JA_PROBLEM_HTML);
        let page = parser.page(false).expect("page should be parsed");

        assert_eq!(page.title, "A - Past ABCs");
        assert_eq!(page.time_limit, 2);
        assert_eq!(page.memory_limit, 1024);
        assert!(page.problem_md.starts_with("## A - Past ABCs"));
    }

    #[test]
    fn page_is_none_without_main_container() {
        let parser = Parser::new("<html><body><p>hello</p></body></html>");
        assert!(parser.page(false).is_none());
        assert!(parser.samples().is_empty());
    }

    #[test]
    fn falls_back_to_default_limits_when_missing() {
        let html = r#"
<html><body>
<div id="main-container">
  <span class="h2">B - No Limits</span>
  <div id="task-statement"><span class="lang-ja"><p>制限の記載なし。</p></span></div>
</div>
</body></html>
"#;

        let parser = Parser::new(html);
        let page = parser.page(false).expect("page should be parsed");

        assert_eq!(page.time_limit, 2);
        assert_eq!(page.memory_limit, 1024);
    }

    #[test]
    fn extracts_limits_in_both_languages() {
        assert_eq!(
            Parser::extract_limits("実行時間制限: 10 sec / メモリ制限: 256 MiB"),
            Some((10, 256))
        );
        assert_eq!(
            Parser::extract_limits("Time Limit: 1 sec / Memory Limit: 2048 MiB"),
            Some((1, 2048))
        );
        assert_eq!(Parser::extract_limits("no limits here"), None);
    }

    const TASKS_PAGE_HTML: &str = r#"
<html><body>
<div id="main-container">
  <table class="table table-bordered table-striped">
    <thead>
      <tr><th></th><th>Task Name</th><th>Time Limit</th><th>Memory Limit</th></tr>
    </thead>
    <tbody>
      <tr>
        <td class="text-center no-break"><a href="/contests/abc350/tasks/abc350_a">A</a></td>
        <td><a href="/contests/abc350/tasks/abc350_a">Past ABCs</a></td>
        <td class="text-right">2 sec</td>
        <td class="text-right">1024 MiB</td>
      </tr>
      <tr>
        <td class="text-center no-break"><a href="/contests/abc350/tasks/abc350_b">B</a></td>
        <td><a href="/contests/abc350/tasks/abc350_b">Dentist Aoki</a></td>
        <td class="text-right">2 sec</td>
        <td class="text-right">1024 MiB</td>
      </tr>
      <tr>
        <td class="text-center no-break"><a href="/contests/other/tasks/other_x">X</a></td>
        <td><a href="/contests/other/tasks/other_x">Other contest</a></td>
        <td class="text-right">2 sec</td>
        <td class="text-right">1024 MiB</td>
      </tr>
    </tbody>
  </table>
</div>
</body></html>
"#;

    #[test]
    fn extracts_screen_names_from_tasks_page() {
        let parser = Parser::new(TASKS_PAGE_HTML);

        let names = parser.task_screen_names("abc350");
        assert_eq!(names, vec!["abc350_a", "abc350_b"]);

        // 別コンテストのリンクは除外される
        assert_eq!(parser.task_screen_names("other"), vec!["other_x"]);
        assert!(parser.task_screen_names("nothing").is_empty());
    }

    const CONTEST_TOP_HTML: &str = r#"
<html><body>
<div id="contest-statement"><span class="lang">
  <span class="lang-ja">
    <h3>コンテスト情報</h3>
    <section><ul><li>コンテスト時間: 100 分</li></ul></section>
    <h3>配点</h3>
    <section>
      <div class="row">
        <div class="span4">
          <table class="table table-responsible table-striped table-bordered">
            <thead><tr><th style="text-align:center">問題</th><th style="text-align:center">点数</th></tr></thead>
            <tbody>
              <tr><td style="text-align:center">A</td><td style="text-align:center">???</td></tr>
              <tr><td style="text-align:center">B</td><td style="text-align:center">200</td></tr>
              <tr><td style="text-align:center">C</td><td style="text-align:center">???</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>
    <h3>ルール</h3>
    <section><ol><li>ルール。</li></ol></section>
  </span>
  <span class="lang-en">
    <h3>Point Values</h3>
    <section>
      <div class="row">
        <div class="span4">
          <table class="table table-responsible table-striped table-bordered">
            <thead><tr><th>Task</th><th>Score</th></tr></thead>
            <tbody>
              <tr><td>A</td><td>???</td></tr>
              <tr><td>B</td><td>200</td></tr>
              <tr><td>C</td><td>???</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>
  </span>
</span></div>
</body></html>
"#;

    #[test]
    fn extracts_task_chars_from_contest_top_page() {
        let parser = Parser::new(CONTEST_TOP_HTML);

        // ja/en両方に同一の表があるため重複は除去される
        assert_eq!(parser.contest_top_task_chars(), vec!["a", "b", "c"]);
    }

    #[test]
    fn top_page_chars_are_empty_without_statement_or_table() {
        let parser = Parser::new("<html><body><p>hello</p></body></html>");
        assert!(parser.contest_top_task_chars().is_empty());

        let no_table = r#"
<html><body>
<div id="contest-statement"><span class="lang">
  <span class="lang-ja"><h3>配点</h3><p>表なし</p></span>
</span></div>
</body></html>
"#;
        let parser = Parser::new(no_table);
        assert!(parser.contest_top_task_chars().is_empty());
    }
}
