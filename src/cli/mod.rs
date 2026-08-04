mod parser;

use anyhow::{bail, Result};
use clap::Parser;
use parser::Cli;

use crate::app;
use crate::content::code;
use crate::persistence::config;
use crate::persistence::config::constants::defaults;
use crate::domain::{CursorStyle, GameMode};
use crate::view::progress as display;
use crate::view::theme;
use crate::content::words;

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
        .unwrap_or_else(|| CursorStyle::Block.token().to_string());

    let config_content = config::toml_parser::get_config()
        .lock()
        .ok()
        .and_then(|c| c.get_content())
        .unwrap_or_default();

    let direct = cli.mode.is_some()
        || cli.time.is_some()
        || !cli.modifiers.is_empty()
        || cli.language.is_some();

    let code_mode = match cli.mode {
        Some(GameMode::Text) => false,
        Some(GameMode::Code) => true,
        None => !direct && GameMode::is_code_content(&config_content),
    };

    if code_mode && !cli.modifiers.is_empty() {
        bail!("`-x/--modifiers` only applies to text mode, not code.");
    }

    let (word_language, code_language) = resolve_languages(&cli, code_mode)?;

    let mode_tokens: Vec<String> = if cli.modifiers.is_empty() {
        config::mode_settings::ModeSettings::new()
            .default_modes
            .iter()
            .map(|m| m.token().to_string())
            .collect()
    } else {
        cli.modifiers
            .iter()
            .map(|m| m.token().to_string())
            .collect()
    };

    let time = cli
        .time
        .or_else(|| {
            config::toml_parser::get_config()
                .lock()
                .ok()
                .and_then(|c| c.get_game())
                .and_then(|g| g.time)
        })
        .unwrap_or(defaults::TIME);

    app::run(
        theme,
        cursor_style,
        word_language,
        mode_tokens,
        time,
        code_mode,
        code_language,
        direct,
    )
}

fn resolve_languages(cli: &Cli, code_mode: bool) -> Result<(String, String)> {
    let config_word = config::language::Language::new().lang;
    let config_code = config::toml_parser::get_config()
        .lock()
        .ok()
        .and_then(|c| c.get_code_language())
        .unwrap_or_else(|| defaults::CODE_LANGUAGE.to_string());

    if code_mode {
        let code_language = match &cli.language {
            Some(lang) => {
                let mut valid = code::available_languages();
                valid.push(defaults::CODE_LANGUAGE.to_string());
                if !valid.contains(lang) {
                    bail!(
                        "Unknown code language '{lang}'. Available: {}.",
                        valid.join(", ")
                    );
                }
                lang.clone()
            }
            None => config_code,
        };
        Ok((config_word, code_language))
    } else {
        let word_language = match &cli.language {
            Some(lang) => {
                let valid = words::available_languages();
                if !valid.contains(lang) {
                    bail!(
                        "Unknown language '{lang}'. Available: {}. \
                         Add your own by placing `<name>.txt` in ~/.local/share/typy/.",
                        valid.join(", ")
                    );
                }
                lang.clone()
            }
            None => config_word,
        };
        Ok((word_language, config_code))
    }
}
