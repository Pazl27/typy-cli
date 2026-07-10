//! Application router: owns the ratatui terminal, sets it up once, and loops
//! over screens (home / settings / stats), launching tests on demand.

use anyhow::{Context, Result};
use crossterm::cursor::SetCursorStyle;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;

use super::game::run_test;
use super::screens::{home, settings as settings_screen, stats as stats_screen};
use super::Tui;
use crate::config::Settings;
use crate::mode::Mode;
use crate::terminal::TestKind;

/// Which screen the router is currently showing.
#[derive(Debug, Clone, Copy)]
pub enum Route {
    Home,
    Settings,
    Stats,
}

/// Everything needed to launch a test.
pub struct TestConfig {
    pub mode: Mode,
    pub kind: TestKind,
}

/// What a screen asks the router to do next.
pub enum Transition {
    Goto(Route),
    StartTest(TestConfig),
    Quit,
}

/// Run the full interactive app, starting on `initial`.
pub fn run(initial: Route) -> Result<()> {
    let mut term = setup_terminal().context("Failed to setup terminal")?;
    let res = run_loop(&mut term, initial);
    restore_terminal().context("Failed to restore terminal")?;
    res
}

/// Run a single test directly (used by the CLI flags), bypassing the home screen.
pub fn run_direct(mode: Mode, kind: TestKind) -> Result<()> {
    let mut term = setup_terminal().context("Failed to setup terminal")?;
    let settings = Settings::load();
    let res = run_test(&mut term, TestConfig { mode, kind }, &settings);
    restore_terminal().context("Failed to restore terminal")?;
    res
}

fn run_loop(term: &mut Tui, mut route: Route) -> Result<()> {
    let mut settings = Settings::load();
    loop {
        match route {
            Route::Home => match home::run(term, &settings)? {
                Transition::StartTest(cfg) => run_test(term, cfg, &settings)?,
                Transition::Goto(r) => route = r,
                Transition::Quit => break,
            },
            Route::Settings => {
                settings_screen::run(term, &mut settings)?;
                route = Route::Home;
            }
            Route::Stats => {
                stats_screen::run(term, &settings)?;
                route = Route::Home;
            }
        }
    }
    Ok(())
}

fn setup_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    let mut out = stdout();
    out.execute(EnterAlternateScreen)?;
    out.execute(cursor::Hide)?;
    let backend = CrosstermBackend::new(out);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    let mut out = stdout();
    out.execute(LeaveAlternateScreen)?;
    out.execute(SetCursorStyle::DefaultUserShape)?;
    out.execute(cursor::Show)?;
    Ok(())
}
