use console::{StyledObject, style};

pub fn error(msg: &str) -> StyledObject<&str> {
    style(msg).red().bold()
}

pub fn warn(msg: &str) -> StyledObject<&str> {
    style(msg).yellow().bold()
}

pub fn success(msg: &str) -> StyledObject<&str> {
    style(msg).green().bold().bright()
}

pub fn info(msg: &str) -> StyledObject<&str> {
    style(msg).blue().bold().bright()
}

pub fn dim(msg: &str) -> StyledObject<&str> {
    style(msg).dim()
}
