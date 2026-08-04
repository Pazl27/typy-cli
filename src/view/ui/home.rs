use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::persistence::config::constants::defaults;
use crate::view::theme::Theme;

use super::constants::text;

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    render_hero(frame, root[0], app, theme);
    render_command_bar(frame, root[1], theme);
}

fn render_hero(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let mut lines: Vec<Line> = text::BANNER
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
    let subtitle = if app.code_mode {
        if app.code_language.is_empty() || app.code_language == defaults::CODE_LANGUAGE {
            text::SUBTITLE_CODING.to_string()
        } else {
            format!("{}  ·  {}", text::SUBTITLE_CODING, app.code_language)
        }
    } else {
        format!("{}  ·  {}s", app.language, app.time)
    };
    lines.push(Line::from(Span::styled(
        subtitle,
        Style::default().fg(theme.missing),
    )));
    if !app.code_mode && app.record > 0 {
        lines.push(Line::from(vec![
            Span::styled(text::LABEL_BEST, Style::default().fg(theme.missing)),
            Span::styled(
                format!("{}{}", app.record, text::SUFFIX_WPM),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(text::HINT_PRESS, Style::default().fg(theme.missing)),
        Span::styled(
            text::HINT_ANY_KEY,
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(text::HINT_TO_START, Style::default().fg(theme.missing)),
    ]));
    if let Some(error) = &app.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default().fg(theme.error),
        )));
    }

    let block = super::centered_vertical(area, lines.len() as u16);
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        block,
    );
}

fn render_command_bar(frame: &mut Frame, area: Rect, theme: &Theme) {
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
        key(text::KEY_SETTINGS),
        label(text::LABEL_SETTINGS),
        key(text::KEY_STATS),
        label(text::LABEL_STATS),
        key(text::KEY_QUIT),
        label(text::LABEL_QUIT),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center),
        area,
    );
}
