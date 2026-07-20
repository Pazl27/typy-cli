use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use super::theme::UiTheme;
use crate::app::App;
use crate::typing::{TypingSession, Word};

pub fn render(frame: &mut Frame, app: &App) {
    let theme = UiTheme::from(&app.theme);
    let Some(session) = app.session.as_ref() else {
        return;
    };
    let area = frame.area();

    let width = area.width.saturating_mul(6) / 10;
    let column = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(area)[1];

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(column);

    render_language(frame, rows[1], &app.language, &theme);
    render_status(frame, rows[3], session, &theme);
    render_words(frame, rows[5], session, &theme);
    render_hint(frame, area, &theme);
}

fn render_language(frame: &mut Frame, area: Rect, language: &str, theme: &UiTheme) {
    let line = Line::from(vec![
        Span::styled(
            language.to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_status(frame: &mut Frame, area: Rect, session: &TypingSession, theme: &UiTheme) {
    let line = Line::from(vec![
        Span::styled(
            format!("{:>2}s", session.remaining_secs()),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ", Style::default()),
        Span::styled(
            format!("{} wpm", live_wpm(session)),
            Style::default().fg(theme.missing),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Left), area);
}

fn render_words(frame: &mut Frame, area: Rect, session: &TypingSession, theme: &UiTheme) {
    let caret = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::REVERSED);

    let mut spans: Vec<Span> = Vec::new();
    for (wi, word) in session.words.iter().enumerate() {
        let is_current = wi == session.cursor_word;
        push_word_spans(&mut spans, word, is_current, caret, theme);
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).wrap(Wrap { trim: false }),
        area,
    );
}

fn push_word_spans(
    spans: &mut Vec<Span<'static>>,
    word: &Word,
    is_current: bool,
    caret: Style,
    theme: &UiTheme,
) {
    let caret_pos = word.typed.len();
    let len = word.target.len().max(word.typed.len());

    for i in 0..len {
        let (ch, base) = if i < word.typed.len() {
            if i < word.target.len() {
                let ok = word.typed[i] == word.target[i];
                (word.target[i], Style::default().fg(if ok { theme.fg } else { theme.error }))
            } else {
                (
                    word.typed[i],
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::UNDERLINED),
                )
            }
        } else {
            (word.target[i], Style::default().fg(theme.missing))
        };

        let style = if is_current && i == caret_pos { caret } else { base };
        spans.push(Span::styled(ch.to_string(), style));
    }

    let space_style = if is_current && caret_pos >= len {
        caret
    } else {
        Style::default()
    };
    spans.push(Span::styled(" ", space_style));
}

fn render_hint(frame: &mut Frame, area: Rect, theme: &UiTheme) {
    let bar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area)[1];
    frame.render_widget(
        Paragraph::new(Span::styled(
            "esc  cancel",
            Style::default().fg(theme.missing),
        ))
        .alignment(Alignment::Center),
        bar,
    );
}

fn live_wpm(session: &TypingSession) -> u32 {
    let wpm = session.stats.wpm();
    if wpm.is_finite() && wpm > 0.0 {
        wpm as u32
    } else {
        0
    }
}
