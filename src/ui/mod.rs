mod home;
mod results;
mod settings;
pub mod theme;
mod typing;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Home => home::render(frame, app),
        Screen::Typing => typing::render(frame, app),
        Screen::Results => results::render(frame, app),
        Screen::Settings => settings::render(frame, app),
    }
}

pub(crate) fn centered_vertical(area: Rect, height: u16) -> Rect {
    let height = height.min(area.height);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    chunks[1]
}
