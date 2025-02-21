mod config;
mod mode;
mod scores;
mod terminal;
mod utils;
mod word_provider;

use anyhow::{Context, Result};
use clap::{App, Arg};
use mode::Mode;

fn main() -> Result<()> {
    let matches = App::new("Typy CLI")
        .version("0.1.0")
        .author("Pazl")
        .about("Monkeytype clone in the terminal for more information check: https://github.com/Pazl27/typy-cli")
        .arg(
            Arg::new("duration")
                .short('t')
                .long("time")
                .help("Sets the duration of the game")
                .default_value("30")
                .takes_value(true),
        )
        .arg(
            Arg::new("stats")
                .short('s')
                .long("stats")
                .help("Shows the stats of the game"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Creates config file if it doesn't exist and opens it"),
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .help("Sets the mode of the game")
                .multiple_values(true)
                .takes_value(true),
        )
        .get_matches();

    let duration_str = matches
        .value_of("duration")
        .context("Failed to get duration value")?;
    let duration: u64 = duration_str.parse().context("Invalid duration value")?;

    let theme = config::theme::ThemeColors::new();

    if matches.is_present("config") {
        utils::create_config()?;
        utils::open_config()?;
        return Ok(());
    }

    if matches.is_present("stats") {
        println!("Stats");
        return Ok(());
    }

    let mut mode_strs: Vec<&str> = matches.values_of("mode").unwrap_or_default().collect();
    mode_strs.is_empty().then(|| mode_strs.clear());

    let mode = Mode::from_str(mode_strs)
        .context("Failed to parse mode")?
        .add_duration(duration);

    terminal::run(mode, theme)?;

    Ok(())
}
