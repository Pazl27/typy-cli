mod config;
mod mode;
mod scores;
mod terminal;
mod word_provider;

use anyhow::{Context, Result};
use clap::Parser;
use mode::Mode;
use terminal::{Route, TestKind};

#[derive(Parser)]
#[command(name = "typy")]
#[command(version = "0.1.0")]
#[command(author = "Pazl27")]
#[command(
    about = "Monkeytype clone in the terminal for more information check: https://github.com/Pazl27/typy-cli"
)]
#[command(long_about = None)]
struct Cli {
    #[arg(short = 't', long = "time", help = "Duration of a timed test (starts a test directly)")]
    time: Option<u64>,

    #[arg(short = 's', long = "stats", help = "Open the detailed stats screen")]
    stats: bool,

    #[arg(short = 'c', long = "config", help = "Create and open the config file in $EDITOR")]
    config: bool,

    #[arg(short = 'm', long = "mode", num_args = 1.., help = "Sets the mode(s) of the test")]
    mode: Vec<String>,

    #[arg(short = 'w', long = "words", help = "Word-count test instead of a timed test (e.g. -w 25)")]
    words: Option<usize>,

    #[arg(long = "zen", help = "Zen mode: no time or word limit")]
    zen: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.config {
        config::create_config()?;
        config::open_config()?;
        return Ok(());
    }

    if cli.stats {
        terminal::run(Route::Stats)?;
        return Ok(());
    }

    // Any test-shaping flag launches a test directly, skipping the home screen.
    let direct = cli.time.is_some() || cli.words.is_some() || cli.zen || !cli.mode.is_empty();

    if direct {
        let duration = cli.time.unwrap_or(30);
        let mut mode_strs: Vec<&str> = cli.mode.iter().map(|s| s.as_str()).collect();
        mode_strs.is_empty().then(|| mode_strs.clear());
        let mode = Mode::from_str(mode_strs)
            .context("Failed to parse mode")?
            .add_duration(duration);

        let kind = if cli.zen {
            TestKind::Zen
        } else if let Some(w) = cli.words {
            TestKind::Words(w)
        } else {
            TestKind::Time(duration)
        };

        terminal::run_direct(mode, kind)?;
    } else {
        terminal::run(Route::Home)?;
    }

    Ok(())
}
