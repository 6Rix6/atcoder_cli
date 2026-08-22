use crate::{atcoder::models::TaskInfo, constants::ATCODER_BASE_URL};

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
        ATCODER_BASE_URL, &contest_id, &task_id
    );

    let submit_url = format!(
        "{}/contests/{}/submit?taskScreenName={}",
        ATCODER_BASE_URL, &contest_id, &task_id
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
