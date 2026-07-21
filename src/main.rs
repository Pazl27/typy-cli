mod app;
mod cli;
mod config;
mod mode;
mod scores;
mod settings;
mod theme;
mod tui;
mod typing;
mod ui;
mod word_provider;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
