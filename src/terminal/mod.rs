mod app;
mod game;
mod keyboard;
mod keybinds;
mod screens;
mod state;
pub(crate) mod ui;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;

/// The concrete ratatui terminal type shared across screens.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub use app::{run, run_direct, Route, TestConfig, Transition};
pub use keybinds::{Action, KeybindPreset};
pub use state::{Keystroke, TestKind};
