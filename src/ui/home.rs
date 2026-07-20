use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::theme::UiTheme;
use crate::app::App;

const BANNER: &[&str] = &[
    "  _                    ",
    " | |_ _   _ _ __  _   _ ",
    " | __| | | | '_ \\| | | |",
    " | |_| |_| | |_) | |_| |",
    "  \\__|\\__, | .__/ \\__, |",
    "      |___/|_|    |___/ ",
];

pub fn render(frame: &mut Frame, app: &App) {
    let theme = UiTheme::from(&app.theme);
    let area = frame.area();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    render_hero(frame, root[0], app, &theme);
    render_command_bar(frame, root[1], &theme);
}

fn render_hero(frame: &mut Frame, area: Rect, app: &App, theme: &UiTheme) {
    let mut lines: Vec<Line> = BANNER
        .iter()
        .map(|row| {
            Line::from(Span::styled(
                *row,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("{}  ·  {}s", app.language, app.time),
        Style::default().fg(theme.missing),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("press ", Style::default().fg(theme.missing)),
        Span::styled(
            "any key",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to start typing", Style::default().fg(theme.missing)),
    ]));

    let block = super::centered_vertical(area, lines.len() as u16);
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        block,
    );
}

fn render_command_bar(frame: &mut Frame, area: Rect, theme: &UiTheme) {
    let key = |k: &'static str| {
        Span::styled(
            k,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
    };
    let label = |t: &'static str| Span::styled(t, Style::default().fg(theme.missing));

    let line = Line::from(vec![
        key(" s "),
        label("settings   "),
        key(" q "),
        label("quit"),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center),
        area,
    );
}
