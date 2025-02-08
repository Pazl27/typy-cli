use std::{fs, process::Command};
use dirs::home_dir;

use crossterm::event::{KeyCode, KeyModifiers};

pub const LINE_LENGTH: i32 = 70;

pub fn close_typy(code: &KeyCode, modifiers: &KeyModifiers) -> Option<()> {
    match code {
        KeyCode::Esc => Some(()),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Some(()),
        _ => None,
    }
}

pub fn calc_size() -> (u16, u16) {
    let (cols, rows) = crossterm::terminal::size().unwrap();
    let x = cols / 2 - (LINE_LENGTH / 2) as u16;
    let y = rows / 2 - 1;

    (x, y)
}

pub fn create_config() {
    if let Some(home_path) = home_dir() {
        let config_dir = home_path.join(".config/typy");
        let config_file = config_dir.join("config.toml");

        if !config_dir.exists() {
            if let Err(e) = fs::create_dir_all(&config_dir) {
                eprintln!("Failed to create config directory: {}", e);
                return;
            }
        }

        if !config_file.exists() {
            if let Err(e) = fs::File::create(&config_file) {
                eprintln!("Failed to create config file: {}", e);
            }
        }
    } else {
        eprintln!("Failed to get home directory");
    }
}

pub fn open_config() {
    if let Some(home_path) = home_dir() {
        let config_dir = home_path.join(".config/typy");
        let config_file = config_dir.join("config.toml");

        if !config_file.exists() {
            eprintln!("Config file doesn't exist");
            return;
        }

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        if let Err(e) = Command::new(editor)
            .arg(config_file)
            .status()
        {
            eprintln!("Failed to open config file: {}", e);
        }


    } else {
        eprintln!("Failed to get home directory");
    }
}
