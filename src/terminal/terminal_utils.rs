use crossterm::event::{KeyCode, KeyModifiers};

use crate::error::{Error, Result};

pub const LINE_LENGTH: i32 = 70;

pub fn close_typy(code: &KeyCode, modifiers: &KeyModifiers) -> Option<()> {
    match code {
        KeyCode::Esc => Some(()),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Some(()),
        _ => None,
    }
}

pub fn calc_middle_for_text() -> Result<(u16, u16)> {
    let (cols, rows) = crossterm::terminal::size()
        .map_err(|e| Error::custom(format!("Failed to get terminal size: {}", e)))?;
    let x = cols / 2 - (LINE_LENGTH / 2) as u16;
    let y = rows / 2 - 1;

    Ok((x, y))
}
