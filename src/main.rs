mod scores;
mod terminal;
mod utils;
mod word_provider;
mod config;

use clap::{App, Arg};

fn main() {
    let matches = App::new("Typy CLI")
        .version("0.1.0")
        .author("Pazl")
        .about("Monkeytype clone in the terminal")
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
                .help("Creates config file if it doesn't exist and opens it")
        )
        .get_matches();

    let duration_str = matches.value_of("duration").unwrap();
    let duration: u64 = duration_str.parse().expect("Invalid duration value");

    let theme = config::theme::ThemeColors::new();

    if matches.is_present("config") {
        utils::create_config();
        utils::open_config();
        return;
    }

    if matches.is_present("stats") {
        println!("Stats");
        return;
    }

    terminal::run(duration, theme);
}
