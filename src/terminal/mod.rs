mod keyboard;
mod game;
mod terminal_utils;

pub use game::{Game, run};
pub use terminal_utils::{calc_middle_for_text, close_typy};
