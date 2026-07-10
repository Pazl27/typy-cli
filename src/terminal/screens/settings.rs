//! Settings screen: edit every setting in-app and persist to config.toml.
//!
//! Fields are addressed by index (`field_*` helpers below) rather than borrowed
//! closures, which keeps the borrow checker happy while still driving a generic
//! list UI. Text/hex fields open an edit buffer; enum/float fields cycle with
//! ←/→.

use anyhow::{Context, Result};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::time::Duration;

use crate::config::cursor_style::CURSOR_STYLES;
use crate::config::Settings;
use crate::terminal::ui::to_rat;
use crate::terminal::Tui;

const LABELS: [&str; 14] = [
    "theme.fg",
    "theme.missing",
    "theme.error",
    "theme.accent",
    "graph.data",
    "graph.title",
    "graph.axis",
    "cursor.style",
    "modes.default",
    "modes.uppercase_chance",
    "modes.punctuation_chance",
    "modes.numbers_chance",
    "language",
    "keybinds.preset",
];
const FIELD_COUNT: usize = 14;

const LANG_IDX: usize = 12;

/// Text-editable fields (hex colors + the default-mode string). The language is
/// a choice over the installed wordlists, so it cycles instead of being typed.
fn is_text(idx: usize) -> bool {
    matches!(idx, 0..=6 | 8)
}

fn cycle<'a>(items: &'a [String], current: &str, dir: i32) -> &'a str {
    if items.is_empty() {
        return "";
    }
    let cur = items.iter().position(|i| i == current).unwrap_or(0) as i32;
    let next = (cur + dir).rem_euclid(items.len() as i32) as usize;
    &items[next]
}

fn display_value(s: &Settings, idx: usize) -> String {
    match idx {
        0 => s.theme_fg.clone(),
        1 => s.theme_missing.clone(),
        2 => s.theme_error.clone(),
        3 => s.theme_accent.clone(),
        4 => s.graph_data.clone(),
        5 => s.graph_title.clone(),
        6 => s.graph_axis.clone(),
        7 => s.cursor_style.clone(),
        8 => s.default_mode.clone(),
        9 => format!("{:.2}", s.uppercase_chance),
        10 => format!("{:.2}", s.punctuation_chance),
        11 => format!("{:.2}", s.numbers_chance),
        12 => s.language.clone(),
        13 => s.keybind_preset.clone(),
        _ => String::new(),
    }
}

fn commit_text(s: &mut Settings, idx: usize, val: String) {
    match idx {
        0 => s.theme_fg = val,
        1 => s.theme_missing = val,
        2 => s.theme_error = val,
        3 => s.theme_accent = val,
        4 => s.graph_data = val,
        5 => s.graph_title = val,
        6 => s.graph_axis = val,
        8 => s.default_mode = val,
        _ => {}
    }
}

fn adjust(s: &mut Settings, idx: usize, dir: i32, langs: &[String]) {
    match idx {
        7 => {
            let cur = CURSOR_STYLES.iter().position(|c| *c == s.cursor_style).unwrap_or(0);
            let next = (cur as i32 + dir).rem_euclid(CURSOR_STYLES.len() as i32) as usize;
            s.cursor_style = CURSOR_STYLES[next].to_string();
        }
        9 => s.uppercase_chance = step(s.uppercase_chance, dir),
        10 => s.punctuation_chance = step(s.punctuation_chance, dir),
        11 => s.numbers_chance = step(s.numbers_chance, dir),
        LANG_IDX => s.language = cycle(langs, &s.language, dir).to_string(),
        13 => {
            s.keybind_preset = if s.keybind_preset == "monkeytype" {
                "10ff".into()
            } else {
                "monkeytype".into()
            };
        }
        _ => {}
    }
}

fn step(v: f32, dir: i32) -> f32 {
    (v + dir as f32 * 0.05).clamp(0.0, 1.0)
}

pub fn run(term: &mut Tui, settings: &mut Settings) -> Result<()> {
    let mut sel = 0usize;
    let mut editing: Option<String> = None;

    // Available wordlists (files in ~/.local/share/typy). Fall back to the
    // current language so the field is never empty.
    let mut langs = crate::word_provider::list_languages();
    if langs.is_empty() {
        langs.push(settings.language.clone());
    }

    loop {
        term.draw(|f| render(f, settings, sel, &editing))
            .context("Failed to draw settings")?;

        if !poll(Duration::from_millis(150)).context("Failed to poll")? {
            continue;
        }
        let Event::Key(KeyEvent {
            code, kind, ..
        }) = read().context("Failed to read event")?
        else {
            continue;
        };
        if kind == KeyEventKind::Release {
            continue;
        }

        if let Some(buf) = editing.as_mut() {
            match code {
                KeyCode::Char(c) => {
                    if buf.len() < 32 {
                        buf.push(c);
                    }
                }
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Enter => {
                    commit_text(settings, sel, editing.take().unwrap());
                }
                KeyCode::Esc => editing = None,
                _ => {}
            }
            continue;
        }

        match code {
            KeyCode::Up | KeyCode::Char('k') => sel = (sel + FIELD_COUNT - 1) % FIELD_COUNT,
            KeyCode::Down | KeyCode::Char('j') => sel = (sel + 1) % FIELD_COUNT,
            KeyCode::Left | KeyCode::Char('h') => adjust(settings, sel, -1, &langs),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => {
                adjust(settings, sel, 1, &langs)
            }
            KeyCode::Enter => {
                if is_text(sel) {
                    editing = Some(display_value(settings, sel));
                } else {
                    adjust(settings, sel, 1, &langs);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => break,
            _ => {}
        }
    }

    settings.save().context("Failed to save settings")?;
    Ok(())
}

fn render(f: &mut ratatui::Frame, settings: &Settings, sel: usize, editing: &Option<String>) {
    let theme = settings.theme();
    let fg = to_rat(theme.fg);
    let accent = to_rat(theme.accent);
    let missing = to_rat(theme.missing);

    let area = f.area();
    let block = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(FIELD_COUNT as u16 + 4),
        Constraint::Min(0),
    ])
    .split(area)[1];
    let cols = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(52),
        Constraint::Min(0),
    ])
    .split(block)[1];
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(FIELD_COUNT as u16),
        Constraint::Length(1),
    ])
    .split(cols);

    f.render_widget(
        Paragraph::new(Span::styled(
            "Settings",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Left),
        rows[0],
    );

    let lines: Vec<Line> = (0..FIELD_COUNT)
        .map(|i| {
            let selected = i == sel;
            let marker = if selected { "▸ " } else { "  " };
            let value = if selected && editing.is_some() {
                format!("{}_", editing.as_ref().unwrap())
            } else {
                display_value(settings, i)
            };
            let base = if selected { accent } else { fg };
            Line::from(Span::styled(
                format!("{}{:<26}{}", marker, LABELS[i], value),
                Style::default().fg(base).add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), rows[1]);

    let hint = if editing.is_some() {
        "type value · enter confirm · esc cancel"
    } else {
        "↑↓/jk move · ←→/hl change · enter edit/toggle · esc save & back"
    };
    f.render_widget(
        Paragraph::new(hint)
            .style(Style::default().fg(missing))
            .alignment(Alignment::Left),
        rows[2],
    );
}
