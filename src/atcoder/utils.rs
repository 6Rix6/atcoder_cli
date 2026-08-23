use crate::{atcoder::models::TaskInfo, constants::ATCODER_BASE_URL};

/// Resolves a contest ID from a plain ID or a contest URL.
///
/// Accepts forms like `abc350`, `https://atcoder.jp/contests/abc350`,
/// or the same URL with query string / trailing path.
pub fn get_contest_id(id_or_url: &str) -> Option<String> {
    let id = if let Some(idx) = id_or_url.find("/contests/") {
        let rest = &id_or_url[idx + "/contests/".len()..];
        rest.split(['?', '/']).next()?
    } else {
        let segment = id_or_url.split('/').rfind(|s| !s.is_empty())?;
        segment.split('?').next()?
    };

    let valid = !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    valid.then(|| id.to_string())
}

pub fn get_task_info(id_or_url: &str) -> Option<TaskInfo> {
    let task_id = id_or_url
        .split('/')
        .rfind(|s| !s.is_empty())?
        .split('?')
        .next()?;

    let valid = task_id
        .split('_')
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric()));

    if !valid {
        return None;
    }

    let parts: Vec<&str> = task_id.split('_').collect();
    let (task_char, contest_parts) = parts.split_last()?;
    if contest_parts.is_empty() {
        return None;
    }

    let contest_id = contest_parts.join("-");

    let url = format!(
        "{}/contests/{}/tasks/{}",
        ATCODER_BASE_URL, contest_id, task_id
    );

    let submit_url = format!(
        "{}/contests/{}/submit?taskScreenName={}",
        ATCODER_BASE_URL, contest_id, task_id
    );

    let info = TaskInfo {
        url,
        submit_url,
        contest_id,
        task_id: task_id.to_string(),
        task_char: task_char.to_string(),
    };

    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_plain_contest_id() {
        assert_eq!(get_contest_id("abc350").as_deref(), Some("abc350"));
        assert_eq!(get_contest_id("practice2").as_deref(), Some("practice2"));
    }

    #[test]
    fn resolves_contest_id_from_url() {
        assert_eq!(
            get_contest_id("https://atcoder.jp/contests/abc350").as_deref(),
            Some("abc350")
        );
        // クエリ・末尾スラッシュ・後続パス
        assert_eq!(
            get_contest_id("https://atcoder.jp/contests/abc350?lang=ja").as_deref(),
            Some("abc350")
        );
        assert_eq!(
            get_contest_id("https://atcoder.jp/contests/abc350/tasks").as_deref(),
            Some("abc350")
        );
        assert_eq!(
            get_contest_id("/contests/tessoku-book/").as_deref(),
            Some("tessoku-book")
        );
    }

    #[test]
    fn accepts_hyphens_and_underscores_in_slugs() {
        assert_eq!(
            get_contest_id("adt_easy_20260824_1").as_deref(),
            Some("adt_easy_20260824_1")
        );
        assert_eq!(
            get_contest_id("c-league2026-1-open").as_deref(),
            Some("c-league2026-1-open")
        );
    }

    #[test]
    fn rejects_invalid_contest_inputs() {
        assert!(get_contest_id("").is_none());
        assert!(get_contest_id("あbc350").is_none());
        assert!(get_contest_id("abc 350").is_none());
        // /contests/ の直後が空
        assert!(get_contest_id("https://atcoder.jp/contests/").is_none());
    }

    #[test]
    fn parses_plain_task_id() {
        let info = get_task_info("abc350_a").expect("valid id");

        assert_eq!(info.task_id, "abc350_a");
        assert_eq!(info.contest_id, "abc350");
        assert_eq!(info.task_char, "a");
        assert_eq!(
            info.url,
            "https://atcoder.jp/contests/abc350/tasks/abc350_a"
        );
        assert_eq!(
            info.submit_url,
            "https://atcoder.jp/contests/abc350/submit?taskScreenName=abc350_a"
        );
    }

    #[test]
    fn parses_task_url_with_query_and_trailing_slash() {
        let url = "https://atcoder.jp/contests/abc350/tasks/abc350_f?lang=ja/";
        let info = get_task_info(url).expect("valid url");

        assert_eq!(info.task_id, "abc350_f");
        assert_eq!(info.contest_id, "abc350");
        assert_eq!(info.task_char, "f");
    }

    #[test]
    fn joins_multiple_contest_parts_with_hyphen() {
        let info = get_task_info("practice2_l").expect("valid id");
        assert_eq!(info.contest_id, "practice2");

        let info = get_task_info("x_y_z").expect("valid id");
        assert_eq!(info.contest_id, "x-y");
        assert_eq!(info.task_char, "z");
    }

    #[test]
    fn rejects_invalid_inputs() {
        // セグメントが1つだけ(task_idとして不完全)
        assert!(get_task_info("abc350").is_none());
        // 空セグメント
        assert!(get_task_info("abc__a").is_none());
        // 英数字以外を含む
        assert!(get_task_info("abc350-a").is_none());
        assert!(get_task_info("あbc_a").is_none());
        assert!(get_task_info("").is_none());
    }
}
