use anyhow::{Context, Result};
use dirs::home_dir;
use std::{fs, io::Write, path::PathBuf, process::Command};
use toml_edit::{value, DocumentMut, Item, Table};

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

fn config_path() -> Result<PathBuf> {
    let home = home_dir().context("Failed to get home directory")?;
    Ok(home.join(".config/typy/config.toml"))
}

pub fn save_settings(language: &str, mode: &str, time: u64) -> Result<()> {
    create_config()?;
    let path = config_path()?;

    let text = fs::read_to_string(&path).context("Failed to read config file")?;
    let mut doc = text
        .parse::<DocumentMut>()
        .context("Failed to parse config file")?;

    set_kv(&mut doc, "language", "lang", value(language));
    set_kv(&mut doc, "modes", "default_mode", value(mode));
    set_kv(&mut doc, "game", "time", value(time as i64));

    fs::write(&path, doc.to_string()).context("Failed to write config file")?;
    Ok(())
}

fn set_kv(doc: &mut DocumentMut, section: &str, key: &str, val: Item) {
    let table = doc.as_table_mut();
    if !table.contains_key(section) {
        table.insert(section, Item::Table(Table::new()));
    }
    if let Some(section_table) = table[section].as_table_mut() {
        section_table.insert(key, val);
    }
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
