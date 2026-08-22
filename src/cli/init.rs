use std::path::PathBuf;

use clap::Args;
use dialoguer::{Confirm, FuzzySelect};

use crate::{
    cli::{self, Outcome, styles::dialog_theme::DialogTheme},
    config::{self, error::ConfigError, models::Config},
    constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME},
    error::AppError,
    languages::LANGUAGES,
};

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(short, long)]
    pub directory: Option<String>,
}

pub async fn init(args: &InitArgs) -> Result<Outcome, AppError> {
    cli::ensure_interactive()?;
    let dir = match args.directory.as_ref() {
        Some(directory) => PathBuf::from(directory),
        None => dirs::config_dir()
            .map(|dir| dir.join(CONFIG_DIR_NAME))
            .ok_or(ConfigError::NoConfigDir)?,
    };
    let path = dir.join(CONFIG_FILE_NAME);

    let names: Vec<&str> = LANGUAGES.iter().map(|l| l.name).collect();

    let selection = FuzzySelect::with_theme(&DialogTheme::default())
        .with_prompt("Choose your favorite programming language")
        .default(0)
        .items(&names)
        .interact_opt()?;

    let Some(index) = selection else {
        println!("Canceled.");
        return Ok(Outcome::Success);
    };

    if path.exists() {
        warn_ln!("A config file already exists at {}.", path.display());

        let proceed = Confirm::with_theme(&DialogTheme::default())
            .with_prompt("Do you want to overwrite it?")
            .default(false)
            .interact()?;

        if !proceed {
            println!("Canceled.");
            return Ok(Outcome::Success);
        }
    }

    let lang = &LANGUAGES[index];
    let config = Config::from(&lang.config);

    let created = config::utils::create_config(config, &dir)?;

    success_ln!("Config created at: {}", created.display());
    Ok(Outcome::Success)
}
