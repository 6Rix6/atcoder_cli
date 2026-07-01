use std::path::PathBuf;

use clap::Args;
use dialoguer::{Confirm, FuzzySelect};

use crate::{
    cli::styles::{self, dialog_theme::DialogTheme},
    config::{self, models::Config},
    constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME},
    languages::LANGUAGES,
};

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(short, long)]
    pub directory: Option<String>,
}

pub async fn init(args: &InitArgs) {
    let dir = args
        .directory
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| {
                    error_ln!("Could not determine config directory.");
                    std::process::exit(1);
                })
                .join(CONFIG_DIR_NAME)
        });
    let path = dir.join(CONFIG_FILE_NAME);

    let names: Vec<&str> = LANGUAGES.iter().map(|l| l.name).collect();

    let selection = FuzzySelect::with_theme(&DialogTheme::default())
        .with_prompt("Choose your favorite programming language")
        .default(0)
        .items(&names)
        .interact_opt()
        .unwrap_or_else(|e| {
            error_ln!("An error occurred: {}", e);
            std::process::exit(1);
        });

    let Some(index) = selection else {
        println!("Canceled.");
        return;
    };

    if path.exists() {
        warn_ln!("A config file already exists at {}.", path.display());

        let proceed = Confirm::with_theme(&DialogTheme::default())
            .with_prompt("Do you want to overwrite it?")
            .default(false)
            .interact()
            .unwrap_or(false);

        if !proceed {
            println!("Canceled.");
            return;
        }
    }

    let lang = &LANGUAGES[index];
    let config = Config::from(&lang.config);

    match config::utils::create_config(config, &dir) {
        Ok(path) => {
            success_ln!("Config created at: {}", path.display());
        }
        Err(e) => {
            error_ln!("Failed to create config file.");
            eprintln!("{}:\n  {}", styles::dim("Details"), e);
        }
    }
}
