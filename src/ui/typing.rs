use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;
use crate::typing::{TypingSession, Word};

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
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

    render_language(frame, rows[1], &app.language, theme);
    render_status(frame, rows[3], session, theme);
    render_words(frame, rows[5], session, theme);
    render_hint(frame, area, theme);
}

fn render_language(frame: &mut Frame, area: Rect, language: &str, theme: &Theme) {
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

fn render_status(frame: &mut Frame, area: Rect, session: &TypingSession, theme: &Theme) {
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

fn render_words(frame: &mut Frame, area: Rect, session: &TypingSession, theme: &Theme) {
    let width = area.width.max(1) as usize;

    let mut lines: Vec<Line> = Vec::new();
    let mut current: Vec<Span> = Vec::new();
    let mut col = 0usize;
    let mut caret_row = 0u16;
    let mut caret_col = 0u16;

    for (wi, word) in session.words.iter().enumerate() {
        let word_len = word.target.len().max(word.typed.len());

        // Greedy word wrap: move the word to the next line if it (plus the
        // separating space) no longer fits. Owning the wrap ourselves means the
        // caret coordinate always matches what is drawn.
        if col > 0 && col + 1 + word_len > width {
            lines.push(Line::from(std::mem::take(&mut current)));
            col = 0;
        }
        if col > 0 {
            current.push(Span::raw(" "));
            col += 1;
        }

        if wi == session.cursor_word {
            caret_row = lines.len() as u16;
            let within = word.typed.len().min(word_len);
            caret_col = (col + within) as u16;
        }

        for i in 0..word_len {
            let (ch, style) = char_style(word, i, theme);
            current.push(Span::styled(ch.to_string(), style));
        }
        col += word_len;
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }

    frame.render_widget(Paragraph::new(lines), area);

    // Place the real terminal cursor at the caret; its shape/blink is set by the
    // app loop. This keeps the character under the caret visible.
    if caret_row < area.height {
        let x = area.x + caret_col.min(area.width.saturating_sub(1));
        let y = area.y + caret_row;
        frame.set_cursor_position((x, y));
    }
}

fn char_style(word: &Word, i: usize, theme: &Theme) -> (char, Style) {
    if i < word.typed.len() {
        if i < word.target.len() {
            let ok = word.typed[i] == word.target[i];
            (
                word.target[i],
                Style::default().fg(if ok { theme.fg } else { theme.error }),
            )
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
    }
}

fn render_hint(frame: &mut Frame, area: Rect, theme: &Theme) {
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
