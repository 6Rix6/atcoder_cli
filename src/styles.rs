use console::{style, StyledObject};

pub fn error(msg: &str) -> StyledObject<&str> {
    style(msg).red().bold()
}

pub fn warn(msg: &str) -> StyledObject<&str> {
    style(msg).yellow().bold()
}

pub fn success(msg: &str) -> StyledObject<&str> {
    style(msg).green().bold().bright()
}
