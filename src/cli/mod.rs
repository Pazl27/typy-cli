mod parser;

use anyhow::{Context, Result};
use clap::Parser;
use parser::Cli;

use crate::app;
use crate::config;
use crate::mode::Mode;
use crate::scores::progress::display;
use crate::theme;

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.config {
        config::create_config()?;
        config::open_config()?;
        return Ok(());
    }

    if cli.stats {
        display::draw()?;
        return Ok(());
    }

    let language = config::language::Language::new().lang;

    let theme_name = config::toml_parser::get_config()
        .lock()
        .ok()
        .and_then(|c| c.get_theme())
        .unwrap_or_else(|| theme::DEFAULT_THEME.to_string());
    let theme = theme::load(&theme_name);

    let cursor_style = config::toml_parser::get_config()
        .lock()
        .ok()
        .and_then(|c| c.get_cursor())
        .unwrap_or_else(|| "block".to_string());

    let config_time = config::toml_parser::get_config()
        .lock()
        .ok()
        .and_then(|c| c.get_game())
        .and_then(|g| g.time);
    let time = cli.time.or(config_time).unwrap_or(30);

    let mode_tokens: Vec<String> = if !cli.mode.is_empty() {
        cli.mode.clone()
    } else {
        config::mode_settings::ModeSettings::new()
            .default_modes
            .iter()
            .map(|m| m.token().to_string())
            .collect()
    };

    Mode::from_str(mode_tokens.iter().map(|s| s.as_str()).collect())
        .context("Failed to parse mode")?;

    let direct = cli.time.is_some() || !cli.mode.is_empty();

    app::run(theme, cursor_style, language, mode_tokens, time, direct)
}
