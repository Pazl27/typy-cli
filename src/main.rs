mod app;
mod cli;
mod content;
mod domain;
mod persistence;
mod session;
mod view;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
