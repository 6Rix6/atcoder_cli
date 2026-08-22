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
    if parts.len() < 2 {
        return None;
    }

    let contest_id = parts[..parts.len() - 1].join("-");
    let task_char = parts.last().unwrap();

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
