#[derive(Debug, Clone)]
pub struct SampleInput {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub url: String,
    pub contest_id: String,
    pub task_id: String,
    pub task_char: String,
}
