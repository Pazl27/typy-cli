//! Keyboard preset handling.
//!
//! Restarting a test is a two-step gesture (like 10FF / monkeytype): press
//! `Tab` to *arm* a restart, then a confirmation key to fire it. The
//! confirmation key differs per preset, which is the whole point of the
//! configurable preset:
//!
//! - **10FF**: `Tab` then `Space`
//! - **monkeytype**: `Tab` then `Enter`
//!
//! The armed flag lives in the game loop; [`KeybindPreset::map`] is a pure
//! function of `(key, armed)` so it stays trivial to reason about and test.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A high-level action derived from a raw key event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Type(char),
    Backspace,
    DeleteWord,
    /// Start a fresh test with new words.
    NewTest,
    Quit,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindPreset {
    /// 10 Fast Fingers: confirm restart with Space.
    TenFF,
    /// monkeytype: confirm restart with Enter.
    Monkeytype,
}

impl KeybindPreset {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "monkeytype" | "monkey" | "mt" => KeybindPreset::Monkeytype,
            _ => KeybindPreset::TenFF,
        }
    }

    /// Human-readable hint for the restart gesture, shown in the footer.
    pub fn restart_hint(&self) -> &'static str {
        match self {
            KeybindPreset::TenFF => "tab then space",
            KeybindPreset::Monkeytype => "tab then enter",
        }
    }

    fn is_confirm(&self, code: KeyCode) -> bool {
        match self {
            KeybindPreset::TenFF => code == KeyCode::Char(' '),
            KeybindPreset::Monkeytype => code == KeyCode::Enter,
        }
    }

    /// Map a raw key event (and the current armed flag) to an [`Action`] and
    /// the next armed flag.
    pub fn map(&self, key: KeyEvent, armed: bool) -> (Action, bool) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            KeyCode::Esc => (Action::Quit, false),
            KeyCode::Char('c') if ctrl => (Action::Quit, false),
            KeyCode::Backspace if ctrl || alt => (Action::DeleteWord, false),
            KeyCode::Backspace => (Action::Backspace, false),
            // Tab arms a restart; the next confirmation key fires it.
            KeyCode::Tab => (Action::None, true),
            code => {
                if armed && self.is_confirm(code) {
                    return (Action::NewTest, false);
                }
                match code {
                    KeyCode::Char(c) => (Action::Type(c), false),
                    _ => (Action::None, false),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn typing_a_letter() {
        let (a, armed) = KeybindPreset::TenFF.map(key(KeyCode::Char('a')), false);
        assert_eq!(a, Action::Type('a'));
        assert!(!armed);
    }

    #[test]
    fn tab_arms_restart() {
        let (a, armed) = KeybindPreset::TenFF.map(key(KeyCode::Tab), false);
        assert_eq!(a, Action::None);
        assert!(armed);
    }

    #[test]
    fn tenff_confirms_with_space() {
        let (a, _) = KeybindPreset::TenFF.map(key(KeyCode::Char(' ')), true);
        assert_eq!(a, Action::NewTest);
        // enter does not confirm under 10FF; it is ignored and disarms
        let (a, _) = KeybindPreset::TenFF.map(key(KeyCode::Enter), true);
        assert_eq!(a, Action::None);
    }

    #[test]
    fn monkeytype_confirms_with_enter() {
        let (a, _) = KeybindPreset::Monkeytype.map(key(KeyCode::Enter), true);
        assert_eq!(a, Action::NewTest);
        // space when armed just types a space under monkeytype
        let (a, _) = KeybindPreset::Monkeytype.map(key(KeyCode::Char(' ')), true);
        assert_eq!(a, Action::Type(' '));
    }

    #[test]
    fn ctrl_backspace_deletes_word() {
        let (a, _) = KeybindPreset::TenFF.map(ctrl(KeyCode::Backspace), false);
        assert_eq!(a, Action::DeleteWord);
    }

    #[test]
    fn esc_and_ctrl_c_quit() {
        assert_eq!(
            KeybindPreset::TenFF.map(key(KeyCode::Esc), false).0,
            Action::Quit
        );
        assert_eq!(
            KeybindPreset::TenFF.map(ctrl(KeyCode::Char('c')), false).0,
            Action::Quit
        );
    }
}
