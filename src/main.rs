mod config;
mod mode;
mod scores;
mod terminal;
mod word_provider;
mod error;

use clap::Parser;
use mode::Mode;
use scores::progress::display;
use error::{Error, Result};

#[derive(Parser)]
#[command(name = "typy")]
#[command(version = "0.1.0")]
#[command(author = "Pazl27")]
#[command(
    about = "Monkeytype clone in the terminal for more information check: https://github.com/Pazl27/typy-cli"
)]
#[command(long_about = None)]
struct Cli {
    #[arg(
        short = 't',
        long = "time",
        default_value = "30",
        help = "Duration of the game"
    )]
    time: u64,

    #[arg(short = 's', long = "stats", help = "Display game stats")]
    stats: bool,

    #[arg(short = 'c', long = "config", help = "Create and open config file")]
    config: bool,

    #[arg(short = 'm', long = "mode", num_args = 1.., help = "Sets the mode of the game")]
    mode: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let duration: u64 = cli.time;

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

    let mut mode_strs: Vec<&str> = cli.mode.iter().map(|s| s.as_str()).collect();
    mode_strs.is_empty().then(|| mode_strs.clear());

    let mode = Mode::from_str(mode_strs)
        .map_err(|_| Error::custom("Failed to parse mode"))?
        .add_duration(duration);

    terminal::run(mode, theme)?;

    Ok(())
}
