use dialoguer::{FuzzySelect, theme::ColorfulTheme};

use crate::languages::LANGUAGES;
use crate::utils::config::{self, Config};

pub fn init() {
    let names: Vec<&str> = LANGUAGES.iter().map(|l| l.name).collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose your favorite programming language")
        .default(0)
        .items(&names)
        .interact_opt()
        .unwrap();

    let Some(index) = selection else {
        println!("Cancelled.");
        return;
    };

    let lang = &LANGUAGES[index];
    let config = Config::from(&lang.config);

    match config::create_config(config) {
        Ok(path) => println!("Config created at: {}", path.display()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
