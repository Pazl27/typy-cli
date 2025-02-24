use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyModifiers};

pub const LINE_LENGTH: i32 = 70;

pub fn close_typy(code: &KeyCode, modifiers: &KeyModifiers) -> Option<()> {
    match code {
        KeyCode::Esc => Some(()),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Some(()),
        _ => None,
    }
}

pub fn calc_middle_for_text() -> Result<(u16, u16)> {
    let (cols, rows) = crossterm::terminal::size().context("Failed to get terminal size")?;
    let x = cols / 2 - (LINE_LENGTH / 2) as u16;
    let y = rows / 2 - 1;

    Ok((x, y))
}
