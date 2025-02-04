use crossterm::event::{KeyCode, KeyModifiers};

pub const LENGTH: i32 = 70;

pub fn close_typy(code: &KeyCode, modifiers: &KeyModifiers) -> Option<()> {
    match code {
        KeyCode::Esc => Some(()),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Some(()),
        _ => None,
    }
}

pub fn calc_size() -> (u16, u16) {
    let (cols, rows) = crossterm::terminal::size().unwrap();
    let x = cols / 2 - (LENGTH / 2) as u16;
    let y = rows / 2 - 1;

    (x, y)
}
