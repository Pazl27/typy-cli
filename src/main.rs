mod app;
mod config;
mod mode;
mod scores;
mod settings;
mod tui;
mod typing;
mod ui;
mod word_provider;

use anyhow::{Context, Result};
use clap::Parser;
use mode::Mode;
use scores::progress::display;

#[derive(Parser)]
#[command(name = "typy")]
#[command(version = "0.1.0")]
#[command(author = "Pazl27")]
#[command(
    about = "Monkeytype clone in the terminal for more information check: https://github.com/Pazl27/typy-cli"
)]
#[command(long_about = None)]
struct Cli {
    #[arg(short = 't', long = "time", help = "Duration of the game")]
    time: Option<u64>,

    #[arg(short = 's', long = "stats", help = "Display game stats")]
    stats: bool,

    #[arg(short = 'c', long = "config", help = "Create and open config file")]
    config: bool,

    #[arg(short = 'm', long = "mode", num_args = 1.., help = "Sets the mode of the game")]
    mode: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let theme = config::theme::ThemeColors::new();

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

    app::run(theme, language, mode_tokens, time)?;

    Ok(())
}
