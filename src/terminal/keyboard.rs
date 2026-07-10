//! Translate high-level [`Action`]s into mutations on the [`TestState`].
//!
//! Only the actions that mutate the *typing* state live here; loop-level
//! actions (`NewTest`, `Quit`) are handled by the game loop, which owns word
//! generation and the exit condition.

use super::keybinds::Action;
use super::state::TestState;

/// Apply a typing action to the state. `elapsed_ms` is the time since the test
/// started running (the loop owns the clock).
pub fn apply_typing(state: &mut TestState, action: Action, elapsed_ms: u128) {
    match action {
        Action::Type(c) => state.type_char(c, elapsed_ms),
        Action::Backspace => state.backspace(),
        Action::DeleteWord => state.delete_word(),
        _ => {}
    }
}
