mod game;
mod keyboard;
mod terminal_utils;

pub use game::{run, Game};
pub use terminal_utils::{calc_middle_for_text, close_typy};
