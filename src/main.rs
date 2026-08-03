mod app;
mod cli;
mod code_provider;
mod config;
mod mode;
mod scores;
mod settings;
mod syntax;
mod theme;
mod tui;
mod typing;
mod ui;
mod word_provider;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
