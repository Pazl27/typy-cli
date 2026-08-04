use clap::{Parser, ValueEnum};

pub(crate) use crate::domain::GameMode;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum Modifier {
    Punctuation,
    Numbers,
    Uppercase,
}

impl Modifier {
    pub(crate) fn mode_type(self) -> crate::domain::ModeType {
        match self {
            Modifier::Punctuation => crate::domain::ModeType::Punctuation,
            Modifier::Numbers => crate::domain::ModeType::Numbers,
            Modifier::Uppercase => crate::domain::ModeType::Uppercase,
        }
    }

    pub(crate) fn token(self) -> &'static str {
        self.mode_type().token()
    }
}

#[derive(Parser)]
#[command(name = "typy")]
#[command(version)]
#[command(author = "Pazl27")]
#[command(about = "A Monkeytype-style typing test for your terminal.")]
#[command(long_about = "typy is a Monkeytype-style typing test for your terminal.

USAGE MODES:
  • Interactive:   typy                     - Open the app (home, settings, stats)
  • Words:         typy -m text -t 60       - A 60s words test, then exit
  • With options:  typy -x punctuation numbers
  • Code:          typy -m code -l rust     - Type a Rust snippet (or -l any)
  • Language:      typy -l german           - Words test in German
  • Stats:         typy -s
  • Config:        typy -c

For more information check: https://github.com/Pazl27/typy-cli")]
pub(crate) struct Cli {
    #[arg(
        short = 'm',
        long = "mode",
        value_name = "MODE",
        help_heading = "Game options",
        help = "What to type: text (words) or code. Default: text."
    )]
    pub(crate) mode: Option<GameMode>,

    #[arg(
        short = 'l',
        long = "language",
        value_name = "LANG",
        help_heading = "Game options",
        help = "Language to use (works for text and code).\nText: english, german, ...   Code: rust, python, go, c, cpp, javascript, any."
    )]
    pub(crate) language: Option<String>,

    #[arg(
        short = 'x',
        long = "modifiers",
        value_name = "MOD",
        num_args = 1..,
        help_heading = "Text options",
        help = "Extra text options (text mode only): punctuation, numbers, uppercase.\nCombine freely, e.g. -x punctuation numbers."
    )]
    pub(crate) modifiers: Vec<Modifier>,

    #[arg(
        short = 't',
        long = "time",
        value_name = "SECONDS",
        help_heading = "Text options",
        help = "Test duration in seconds (text mode)."
    )]
    pub(crate) time: Option<u64>,

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
