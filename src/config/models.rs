use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Directory-layout customization for `acc setup`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SetupConfig {
    /// Directory under which the contest directory is created. Supports a
    /// leading `~`. Overridden by the `--directory` flag.
    pub parent: Option<String>,

    /// Path pattern of each problem directory, relative to the contest
    /// directory. Supports `${CONTEST_ID}` / `${TASK_ID}` / `${TASK_CHAR}`.
    pub dir: Option<String>,

    /// Files to create inside each problem directory. Patterns support the
    /// same variables as `dir`.
    pub file: Option<Vec<String>>,

    /// Inline source templates keyed by file extension. Files whose extension
    /// has no entry fall back to the language preset or an empty file.
    pub template: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub compiler: Option<String>,
    pub compiler_flags: Option<Vec<String>>,
    pub run: String,
    pub run_flags: Option<Vec<String>>,
    #[serde(default)]
    pub setup: Option<SetupConfig>,
}

pub struct ConfigRef {
    pub compiler: Option<&'static str>,
    pub compiler_flags: Option<&'static [&'static str]>,
    pub run: &'static str,
    pub run_flags: Option<&'static [&'static str]>,
}

impl From<&ConfigRef> for Config {
    fn from(r: &ConfigRef) -> Self {
        Config {
            compiler: r.compiler.map(str::to_string),
            compiler_flags: r
                .compiler_flags
                .map(|f| f.iter().map(|s| s.to_string()).collect()),
            run: r.run.to_string(),
            run_flags: r
                .run_flags
                .map(|f| f.iter().map(|s| s.to_string()).collect()),
            setup: None,
        }
    }
}
