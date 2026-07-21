use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Game};
use crate::theme::Theme;
use crate::typing::highlight::{self};
use crate::typing::{CodeSession, TypingSession, Word};

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let Some(session) = app.session.as_ref() else {
        return;
    };
    match session {
        Game::Words(s) => render_words_mode(frame, app, s, theme),
        Game::Code(s) => render_code_mode(frame, s, theme),
    }
}

fn render_words_mode(frame: &mut Frame, app: &App, session: &TypingSession, theme: &Theme) {
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
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(column);

    render_language(frame, rows[1], &app.language, theme);
    render_status(frame, rows[3], session, theme);
    render_words(frame, rows[5], session, theme);
    render_hint(frame, area, theme);
}

fn render_code_mode(frame: &mut Frame, session: &CodeSession, theme: &Theme) {
    let area = frame.area();

    let (line_count, max_line) = code_dimensions(&session.target);
    let width = (max_line as u16 + 1).clamp(20, area.width.saturating_sub(4).max(20));
    let code_height = (line_count as u16).clamp(1, area.height.saturating_sub(6).max(1));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(code_height),
            Constraint::Min(0),
        ])
        .split(area);

    let code_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(rows[3])[1];

    render_code_status(frame, rows[1], session, theme);
    render_code(frame, code_area, session, theme);
    render_code_footer(frame, area, session, theme);
}

fn code_dimensions(target: &[char]) -> (usize, usize) {
    let mut lines = 1;
    let mut max = 0;
    let mut current = 0;
    for &c in target {
        if c == '\n' {
            lines += 1;
            max = max.max(current);
            current = 0;
        } else {
            current += 1;
        }
    }
    max = max.max(current);
    (lines, max)
}

fn render_code_status(frame: &mut Frame, area: Rect, session: &CodeSession, theme: &Theme) {
    let file_name = session.path.rsplit('/').next().unwrap_or(&session.path);
    let line = Line::from(vec![
        Span::styled(
            file_name.to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  ·  {}  ·  ", session.language), Style::default().fg(theme.missing)),
        Span::styled(
            format!("{} wpm", session.live_wpm()),
            Style::default().fg(theme.missing),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_code_footer(frame: &mut Frame, area: Rect, session: &CodeSession, theme: &Theme) {
    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area)[1];

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(12)])
        .split(bottom);

    frame.render_widget(
        Paragraph::new(Span::styled(
            session.url.clone(),
            Style::default().fg(theme.missing),
        ))
        .alignment(Alignment::Left),
        cols[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            "esc  cancel",
            Style::default().fg(theme.missing),
        ))
        .alignment(Alignment::Right),
        cols[1],
    );
}

fn render_code(frame: &mut Frame, area: Rect, session: &CodeSession, theme: &Theme) {
    let mut lines_chars: Vec<Vec<char>> = vec![Vec::new()];
    let mut line_starts: Vec<usize> = vec![0];
    for (i, &ch) in session.target.iter().enumerate() {
        if ch == '\n' {
            lines_chars.push(Vec::new());
            line_starts.push(i + 1);
        } else {
            lines_chars.last_mut().unwrap().push(ch);
        }
    }

    let mut caret_line = lines_chars.len().saturating_sub(1);
    let mut caret_col = 0u16;
    for (k, &start) in line_starts.iter().enumerate() {
        let end = if k + 1 < line_starts.len() {
            line_starts[k + 1] - 1
        } else {
            session.target.len()
        };
        if session.cursor >= start && session.cursor <= end {
            caret_line = k;
            caret_col = (session.cursor - start) as u16;
            break;
        }
    }

    let mut rendered: Vec<Line> = Vec::new();
    for (k, chars) in lines_chars.iter().enumerate() {
        let start = line_starts[k];
        let tokens = highlight::tokens_for_line(chars, &session.language);
        let mut spans: Vec<Span> = Vec::new();
        for (j, &ch) in chars.iter().enumerate() {
            let idx = start + j;
            let syntax = highlight::token_color(tokens[j], theme);
            spans.push(Span::styled(ch.to_string(), code_char_style(session, idx, syntax, theme)));
        }
        rendered.push(Line::from(spans));
    }

    let h = area.height as usize;
    let total = rendered.len();
    let start_line = if total <= h {
        0
    } else {
        caret_line.saturating_sub(h / 2).min(total - h)
    };
    let visible: Vec<Line> = rendered.into_iter().skip(start_line).take(h).collect();
    frame.render_widget(Paragraph::new(visible), area);

    let cursor_row = caret_line.saturating_sub(start_line) as u16;
    if cursor_row < area.height {
        let x = area.x + caret_col.min(area.width.saturating_sub(1));
        let y = area.y + cursor_row;
        frame.set_cursor_position((x, y));
    }
}

fn code_char_style(
    session: &CodeSession,
    idx: usize,
    syntax: ratatui::style::Color,
    theme: &Theme,
) -> Style {
    if !session.typeable[idx] {
        return Style::default().fg(theme.missing);
    }
    if idx < session.cursor {
        match session.typed[idx] {
            Some(c) if c == session.target[idx] => Style::default().fg(syntax),
            _ => Style::default().fg(theme.error),
        }
    } else {
        Style::default().fg(theme.missing)
    }
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
    let mut caret_line = 0usize;
    let mut caret_col = 0u16;

    for (wi, word) in session.words.iter().enumerate() {
        let word_len = word.target.len().max(word.typed.len());

        if col > 0 && col + 1 + word_len > width {
            lines.push(Line::from(std::mem::take(&mut current)));
            col = 0;
        }
        if col > 0 {
            current.push(Span::raw(" "));
            col += 1;
        }

        if wi == session.cursor_word {
            caret_line = lines.len();
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

    let window_start = caret_line.saturating_sub(1);
    let visible: Vec<Line> = lines
        .into_iter()
        .skip(window_start)
        .take(area.height as usize)
        .collect();
    frame.render_widget(Paragraph::new(visible), area);

    let cursor_row = (caret_line - window_start) as u16;
    if cursor_row < area.height {
        let x = area.x + caret_col.min(area.width.saturating_sub(1));
        let y = area.y + cursor_row;
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
    session.live_wpm()
}
