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

pub struct Tui {
    pub terminal: Terminal<Backend>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(Self { terminal })
    }

    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        Self::install_panic_hook();
        self.terminal.clear()?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        Self::restore()
    }

    fn restore() -> Result<()> {
        disable_raw_mode()?;
        execute!(
            stdout(),
            LeaveAlternateScreen,
            cursor::SetCursorStyle::DefaultUserShape,
            cursor::Show
        )?;
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
