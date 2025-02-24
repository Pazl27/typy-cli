use crate::error::{Error, Result};
use dirs::home_dir;
use std::{fs, io::Write, process::Command};

pub fn create_config() -> Result<()> {
    if let Some(home_path) = home_dir() {
        let config_dir = home_path.join(".config/typy");
        let config_file = config_dir.join("config.toml");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| Error::custom(format!("Failed to create config directory: {}", e)))?;
        }

        if !config_file.exists() {
            let mut file = fs::File::create(&config_file).map_err(|e| Error::custom(format!("Failed to create config file: {}", e)))?;

            file.write_all(b"# For more information about the configuration check:\n# https://github.com/Pazl27/typy-cli?tab=readme-ov-file#configuration")
                .map_err(|e| Error::custom(format!("Failed to write to config file: {}", e)))?;
        }
    } else {
        eprintln!("Failed to get home directory");
    }
    Ok(())
}

pub fn open_config() -> Result<()> {
    if let Some(home_path) = home_dir() {
        let config_dir = home_path.join(".config/typy");
        let config_file = config_dir.join("config.toml");

        if !config_file.exists() {
            eprintln!("Config file doesn't exist");
            return Ok(());
        }

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        Command::new(editor.clone())
            .arg(config_file)
            .status()
            .map_err(|e| Error::custom(format!("Failed to open config file with editor: {}", e)))?;
    } else {
        eprintln!("Failed to get home directory");
    }
    Ok(())
}
