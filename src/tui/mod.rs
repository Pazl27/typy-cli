pub mod events;

use std::io::{stdout, Stdout};

use anyhow::Result;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub type Backend = CrosstermBackend<Stdout>;

/// Owns the terminal and is responsible for entering/leaving the alternate
/// screen and raw mode. All the low-level terminal wrangling lives here so the
/// rest of the app never has to think about it.
pub struct Tui {
    pub terminal: Terminal<Backend>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(Self { terminal })
    }

    /// Switch the terminal into TUI mode (raw + alternate screen) and install a
    /// panic hook so a crash can never leave the user's terminal broken.
    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        Self::install_panic_hook();
        self.terminal.clear()?;
        Ok(())
    }

    /// Restore the terminal to its original state.
    pub fn exit(&mut self) -> Result<()> {
        Self::restore()
    }

    fn restore() -> Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
        Ok(())
    }

    fn install_panic_hook() {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = Self::restore();
            hook(info);
        }));
    }
}
