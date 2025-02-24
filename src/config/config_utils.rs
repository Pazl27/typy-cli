use anyhow::{Context, Result};
use dirs::home_dir;
use std::{fs, io::Write, process::Command};

pub fn create_config() -> Result<()> {
    if let Some(home_path) = home_dir() {
        let config_dir = home_path.join(".config/typy");
        let config_file = config_dir.join("config.toml");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        if !config_file.exists() {
            let mut file =
                fs::File::create(&config_file).context("Failed to create config file")?;

            file.write_all(b"# For more information about the configuration check:\n# https://github.com/Pazl27/typy-cli?tab=readme-ov-file#configuration")
                .context("Failed to write to config file")?;
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
            .with_context(|| format!("Failed to open config file with editor: {}", editor))?;
    } else {
        eprintln!("Failed to get home directory");
    }
    Ok(())
}
