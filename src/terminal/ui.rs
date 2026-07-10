//! Immediate-mode rendering of a typing test with `ratatui`.
//!
//! Everything is redrawn from [`TestState`] each frame, so there is no manual
//! cursor bookkeeping. The target text is wrapped into visual lines at word
//! boundaries and a small window of lines is shown around the cursor, which
//! gives scrolling for free and removes the old 3-line limit.

use ratatui::layout::{Alignment, Constraint, Layout, Position};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::theme::ThemeColors;

use super::state::{CharState, TestState};

/// Number of target lines shown at once.
const VISIBLE_LINES: usize = 3;
/// Maximum text column width.
const MAX_WIDTH: usize = 80;

/// Convert a `crossterm` color (used by the config/theme layer) to a `ratatui`
/// color (used by the renderer).
pub(crate) fn to_rat(c: crossterm::style::Color) -> ratatui::style::Color {
    use crossterm::style::Color as C;
    use ratatui::style::Color as R;
    match c {
        C::Rgb { r, g, b } => R::Rgb(r, g, b),
        C::AnsiValue(v) => R::Indexed(v),
        C::Reset => R::Reset,
        C::Black => R::Black,
        C::DarkGrey => R::DarkGray,
        C::Red => R::Red,
        C::DarkRed => R::LightRed,
        C::Green => R::Green,
        C::DarkGreen => R::LightGreen,
        C::Yellow => R::Yellow,
        C::DarkYellow => R::LightYellow,
        C::Blue => R::Blue,
        C::DarkBlue => R::LightBlue,
        C::Magenta => R::Magenta,
        C::DarkMagenta => R::LightMagenta,
        C::Cyan => R::Cyan,
        C::DarkCyan => R::LightCyan,
        C::White => R::White,
        C::Grey => R::Gray,
    }
}

/// Wrap the flattened target into `[start, end)` index ranges, breaking after a
/// space so words stay intact. Every index belongs to exactly one line.
fn wrap_lines(target: &[char], width: usize) -> Vec<(usize, usize)> {
    let n = target.len();
    if width == 0 || n == 0 {
        return vec![(0, n)];
    }
    let mut lines = Vec::new();
    let mut start = 0;
    while start < n {
        let mut end = (start + width).min(n);
        if end < n {
            if let Some(pos) = (start..end).rev().find(|&k| target[k] == ' ') {
                if pos > start {
                    end = pos + 1; // keep the space on this line
                }
            }
        }
        lines.push((start, end));
        start = end;
    }
    lines
}

pub fn render(f: &mut Frame, state: &TestState, theme: &ThemeColors, header: &str, footer: &str) {
    let area = f.area();

    let fg = to_rat(theme.fg);
    let missing = to_rat(theme.missing);
    let error = to_rat(theme.error);
    let accent = to_rat(theme.accent);

    let width = (area.width.saturating_sub(4) as usize).clamp(20, MAX_WIDTH);
    let lines = wrap_lines(state.target(), width);
    let cursor = state.cursor();

    let cursor_line = lines
        .iter()
        .position(|&(s, e)| cursor >= s && cursor < e)
        .unwrap_or_else(|| lines.len().saturating_sub(1));

    let visible = VISIBLE_LINES.min(lines.len());
    let max_first = lines.len().saturating_sub(visible);
    let first = cursor_line.saturating_sub(1).min(max_first);

    let mut body: Vec<Line> = Vec::new();
    for &(ls, le) in &lines[first..first + visible] {
        let mut spans = Vec::new();
        for i in ls..le {
            let ch = state.target()[i];
            let cstate = state.char_state(i);
            let style = match cstate {
                CharState::Correct => Style::default().fg(fg),
                CharState::Incorrect => Style::default().fg(error),
                CharState::Untyped => Style::default().fg(missing),
            };
            // An incorrectly typed space would be invisible; show a marker.
            let display = if ch == ' ' && cstate == CharState::Incorrect {
                '·'
            } else {
                ch
            };
            spans.push(Span::styled(display.to_string(), style));
        }
        body.push(Line::from(spans));
    }

    // Vertically center a content block of: header + blank + body + blank + footer.
    let content_h = visible as u16 + 4;
    let vchunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(content_h),
        Constraint::Min(0),
    ])
    .split(area);

    let block_w = (width as u16 + 2).min(vchunks[1].width);
    let hchunks = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(block_w),
        Constraint::Min(0),
    ])
    .split(vchunks[1]);
    let col = hchunks[1];

    let rows = Layout::vertical([
        Constraint::Length(1),                // header
        Constraint::Length(1),                // spacer
        Constraint::Length(visible as u16),   // body
        Constraint::Length(1),                // spacer
        Constraint::Length(1),                // footer
    ])
    .split(col);

    f.render_widget(
        Paragraph::new(header)
            .style(Style::default().fg(accent))
            .alignment(Alignment::Left),
        rows[0],
    );
    let body_area = rows[2];
    f.render_widget(Paragraph::new(body), body_area);
    f.render_widget(
        Paragraph::new(footer)
            .style(Style::default().fg(missing))
            .alignment(Alignment::Left),
        rows[4],
    );

    // Place the real terminal cursor at the typing position so the configured
    // cursor style (blinking bar, block, …) is used.
    if cursor_line >= first && cursor_line < first + visible {
        let (ls, _) = lines[cursor_line];
        let col = (cursor - ls) as u16;
        let cx = body_area.x + col;
        let cy = body_area.y + (cursor_line - first) as u16;
        if cx < body_area.x + body_area.width && cy < body_area.y + body_area.height {
            f.set_cursor_position(Position::new(cx, cy));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_breaks_at_spaces() {
        let t: Vec<char> = "aa bb cc dd".chars().collect();
        let lines = wrap_lines(&t, 6);
        // "aa bb " then "cc dd"
        assert_eq!(lines[0], (0, 6));
        assert_eq!(&t[lines[0].0..lines[0].1], &"aa bb ".chars().collect::<Vec<_>>()[..]);
        // every index covered exactly once
        assert_eq!(lines.last().unwrap().1, t.len());
    }

    #[test]
    fn wrap_hard_splits_long_word() {
        let t: Vec<char> = "abcdefghij".chars().collect();
        let lines = wrap_lines(&t, 4);
        assert_eq!(lines[0], (0, 4));
        assert_eq!(lines.last().unwrap().1, t.len());
    }
}
