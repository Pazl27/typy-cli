mod terminal;
mod word_provider;

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
                .value_name("WORD_LIST")
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
        .get_matches();


    let _duration = matches.value_of("duration").unwrap();

    if matches.is_present("stats") {
        println!("Stats");
        return;
    }

    terminal::run();
}
