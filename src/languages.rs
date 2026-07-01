use crate::config::models::ConfigRef;

pub struct Language {
    pub name: &'static str,
    pub config: ConfigRef,
}

pub const LANGUAGES: &[Language] = &[
    Language {
        name: "C++ (g++)",
        config: ConfigRef {
            compiler: Some("g++"),
            compiler_flags: Some(&["main.cpp", "-O2", "-o", "solution"]),
            run: "./solution",
            run_flags: None,
        },
    },
    Language {
        name: "Python",
        config: ConfigRef {
            compiler: None,
            compiler_flags: None,
            run: "python3",
            run_flags: Some(&["main.py", "-u"]),
        },
    },
    Language {
        name: "Rust",
        config: ConfigRef {
            compiler: Some("cargo build --release"),
            compiler_flags: None,
            run: "./target/release/solution",
            run_flags: None,
        },
    },
];
