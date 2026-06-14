use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub compiler: Option<String>,
    pub compiler_flags: Option<Vec<String>>,
    pub run: String,
    pub run_flags: Option<Vec<String>>,
    pub cookie: Option<String>,
}

pub struct ConfigRef {
    pub compiler: Option<&'static str>,
    pub compiler_flags: Option<&'static [&'static str]>,
    pub run: &'static str,
    pub run_flags: Option<&'static [&'static str]>,
    pub cookie: Option<&'static str>,
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
            cookie: r.cookie.map(str::to_string),
        }
    }
}

