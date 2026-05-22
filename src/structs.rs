use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub compiler: String,
    pub flags: Vec<String>,
    pub run: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Url of task
    #[arg(short,long)]
    pub task: String,

    /// Override compiler (Optional)
    #[arg(short, long)]
    pub compiler: Option<String>,

    /// Override compiler flags (Optional)
    #[arg(short, long)]
    pub flags: Option<Vec<String>>,

    /// Override run command
    #[arg(short,long)]
    pub run: Option<String>
}
