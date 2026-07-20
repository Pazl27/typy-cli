mod home;
mod results;
pub mod theme;
mod typing;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Screen};
use theme::UiTheme;

/// Top-level render dispatch. Pure function of `App`.
pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Home => home::render(frame, app),
        Screen::Typing => typing::render(frame, app),
        Screen::Results => results::render(frame, app),
        Screen::Settings => placeholder(frame, app, "settings", "coming soon"),
    }
}

/// Temporary centered message used for screens not yet implemented.
fn placeholder(frame: &mut Frame, app: &App, title: &str, subtitle: &str) {
    let theme = UiTheme::from(&app.theme);
    let area = frame.area();
    let lines = vec![
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(subtitle, Style::default().fg(theme.missing))),
        Line::from(""),
        Line::from(Span::styled(
            "esc  back",
            Style::default().fg(theme.missing),
        )),
    ];
    let block = centered_vertical(area, lines.len() as u16);
    frame.render_widget(
        Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center),
        block,
    );
}

/// Return a full-width rect vertically centered within `area` with `height`
/// rows (clamped to the area).
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
