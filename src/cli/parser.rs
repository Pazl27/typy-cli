use clap::Parser;

#[derive(Parser)]
#[command(name = "typy")]
#[command(version)]
#[command(author = "Pazl27")]
#[command(about = "A Monkeytype-style typing test for your terminal.")]
#[command(long_about = "typy is a Monkeytype-style typing test for your terminal.

USAGE MODES:
  • Interactive:   typy                     - Open the app (home, settings, stats)
  • Quick run:     typy -t 60               - Start a 60s test right away, then exit
  • With mode:     typy -m punctuation      - Start a test with the given mode(s)
  • Stats:         typy -s                  - Show statistics for your past games
  • Config:        typy -c                  - Create and open the config file

For more information check: https://github.com/Pazl27/typy-cli")]
pub(crate) struct Cli {
    #[arg(
        short = 't',
        long = "time",
        value_name = "SECONDS",
        help_heading = "Game options",
        help = "Duration of the test in seconds.\nStarts a test immediately and exits when it finishes."
    )]
    pub(crate) time: Option<u64>,

    #[arg(
        short = 'm',
        long = "mode",
        value_name = "MODE",
        num_args = 1..,
        help_heading = "Game options",
        help = "Mode(s) to play: normal, uppercase, punctuation.\nStarts a test immediately."
    )]
    pub(crate) mode: Vec<String>,

    #[arg(
        short = 's',
        long = "stats",
        help_heading = "Utility",
        help = "Display statistics for your past games."
    )]
    pub(crate) stats: bool,

    #[arg(
        short = 'c',
        long = "config",
        help_heading = "Utility",
        help = "Create the config file if missing and open it in $EDITOR."
    )]
    pub(crate) config: bool,
}
