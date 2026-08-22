use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SampleInput {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct TaskPage {
    pub title: String,
    pub time_limit: u32,
    pub memory_limit: u32,
    pub problem_md: String,
}

#[derive(Debug, Clone)]
pub struct TaskData {
    pub page: TaskPage,
    pub samples: Vec<SampleInput>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub url: String,
    pub submit_url: String,
    pub contest_id: String,
    pub task_id: String,
    pub task_char: String,
}

impl TaskInfo {
    pub fn to_map(&self) -> HashMap<&'static str, &str> {
        let mut map: HashMap<&'static str, &str> = HashMap::new();
        map.insert("CONTEST_ID", &self.contest_id);
        map.insert("TASK_ID", &self.task_id);
        map.insert("TASK_CHAR", &self.task_char);

        map
    }
}

#[derive(Debug)]
pub enum TaskLanguage {
    Japanese,
    English,
}

impl TaskLanguage {
    pub fn code(&self) -> &str {
        match self {
            Self::Japanese => "ja",
            Self::English => "en",
        }
    }
}
