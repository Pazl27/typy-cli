//! Pure, terminal-agnostic model of a typing test.
//!
//! The test is represented as two flat character vectors that stay aligned by
//! index: `target` (the text to reproduce, words joined by a single space) and
//! `typed` (what the user has entered so far). The cursor is simply
//! `typed.len()`. This makes correction trivial (`backspace` = `pop`), restart
//! trivial (clear `typed`), and per-keystroke statistics trivial (append a
//! [`Keystroke`] on every key).
//!
//! Timing is kept *out* of this module so it stays deterministic and unit
//! testable: callers pass the elapsed milliseconds since the test started.

/// What ends a test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestKind {
    /// Fixed duration in seconds.
    Time(u64),
    /// Fixed number of words.
    Words(usize),
    /// No limit; runs until the user restarts or quits.
    Zen,
}

/// Lifecycle of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Nothing typed yet; the clock has not started.
    Idle,
    /// At least one key pressed; the clock is running.
    Running,
    /// The test is over (time up or all words typed).
    Finished,
}

/// A single keystroke, logged for statistics. Never removed on backspace, so it
/// captures the true history of the attempt (including corrected mistakes).
#[derive(Debug, Clone, Copy)]
pub struct Keystroke {
    /// Milliseconds since the test started running.
    pub t_ms: u128,
    /// The character that was expected at that position (`None` past the end).
    pub target: Option<char>,
    /// The character the user actually pressed. Retained for future analysis
    /// (e.g. "pressed X instead of Y" substitution heatmaps).
    #[allow(dead_code)]
    pub typed: char,
    /// Whether `typed` matched `target`.
    pub correct: bool,
}

/// Rendering state of a single target character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    /// Correctly typed.
    Correct,
    /// Typed but wrong.
    Incorrect,
    /// Not yet reached.
    Untyped,
}

pub struct TestState {
    target: Vec<char>,
    typed: Vec<char>,
    keys: Vec<Keystroke>,
    kind: TestKind,
    status: Status,
}

impl TestState {
    pub fn new(target: Vec<char>, kind: TestKind) -> Self {
        TestState {
            target,
            typed: Vec::new(),
            keys: Vec::new(),
            kind,
            status: Status::Idle,
        }
    }

    // --- accessors -------------------------------------------------------

    pub fn target(&self) -> &[char] {
        &self.target
    }

    pub fn typed(&self) -> &[char] {
        &self.typed
    }

    pub fn keys(&self) -> &[Keystroke] {
        &self.keys
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// Cursor position: index of the next character to type.
    pub fn cursor(&self) -> usize {
        self.typed.len()
    }

    /// Rendering state of the target character at `i`.
    pub fn char_state(&self, i: usize) -> CharState {
        match self.typed.get(i) {
            None => CharState::Untyped,
            Some(&c) if Some(c) == self.target.get(i).copied() => CharState::Correct,
            Some(_) => CharState::Incorrect,
        }
    }

    /// True once every target character has been typed (used by word mode).
    pub fn is_complete(&self) -> bool {
        !self.target.is_empty() && self.typed.len() >= self.target.len()
    }

    // --- mutations -------------------------------------------------------

    /// Register a typed character. `elapsed_ms` is the time since the test
    /// started (the caller owns the clock). No-op once finished or when the
    /// target is already fully typed (overflow is clamped).
    pub fn type_char(&mut self, c: char, elapsed_ms: u128) {
        if self.status == Status::Finished || self.cursor() >= self.target.len() {
            return;
        }
        if self.status == Status::Idle {
            self.status = Status::Running;
        }

        let target = self.target.get(self.cursor()).copied();
        let correct = target == Some(c);
        self.keys.push(Keystroke {
            t_ms: elapsed_ms,
            target,
            typed: c,
            correct,
        });
        self.typed.push(c);

        if self.kind_is_words() && self.is_complete() {
            self.status = Status::Finished;
        }
    }

    /// Delete the last typed character.
    pub fn backspace(&mut self) {
        if self.status == Status::Finished {
            return;
        }
        self.typed.pop();
    }

    /// Delete back to the start of the current word (past the previous space in
    /// the target). Mirrors Ctrl+Backspace in monkeytype / 10FF.
    pub fn delete_word(&mut self) {
        if self.status == Status::Finished {
            return;
        }
        let mut i = self.typed.len();
        while i > 0 && self.target.get(i - 1) == Some(&' ') {
            i -= 1;
        }
        while i > 0 && self.target.get(i - 1) != Some(&' ') {
            i -= 1;
        }
        self.typed.truncate(i);
    }

    /// Restart with the *same* target text.
    pub fn restart_same(&mut self) {
        self.typed.clear();
        self.keys.clear();
        self.status = Status::Idle;
    }

    /// Restart with a *new* target text.
    pub fn new_test(&mut self, target: Vec<char>) {
        self.target = target;
        self.restart_same();
    }

    /// Mark the test finished (called by the loop when the timer expires).
    pub fn finish(&mut self) {
        self.status = Status::Finished;
    }

    /// Append more target characters (time / zen modes top up the pool so the
    /// user never runs out of text to type).
    pub fn extend_target(&mut self, more: &[char]) {
        self.target.extend_from_slice(more);
    }

    fn kind_is_words(&self) -> bool {
        matches!(self.kind, TestKind::Words(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(text: &str) -> TestState {
        TestState::new(text.chars().collect(), TestKind::Time(30))
    }

    #[test]
    fn typing_correct_and_incorrect_chars() {
        let mut s = state("cat");
        s.type_char('c', 0);
        s.type_char('x', 10); // wrong
        s.type_char('t', 20);
        assert_eq!(s.char_state(0), CharState::Correct);
        assert_eq!(s.char_state(1), CharState::Incorrect);
        assert_eq!(s.char_state(2), CharState::Correct);
        assert_eq!(s.cursor(), 3);
        assert_eq!(s.keys().len(), 3);
        assert!(!s.keys()[1].correct);
    }

    #[test]
    fn status_starts_on_first_key() {
        let mut s = state("cat");
        assert_eq!(s.status(), Status::Idle);
        s.type_char('c', 0);
        assert_eq!(s.status(), Status::Running);
    }

    #[test]
    fn backspace_pops_typed_but_keeps_keylog() {
        let mut s = state("cat");
        s.type_char('c', 0);
        s.type_char('x', 10);
        s.backspace();
        assert_eq!(s.cursor(), 1);
        assert_eq!(s.char_state(1), CharState::Untyped);
        // keystroke history is preserved for stats
        assert_eq!(s.keys().len(), 2);
    }

    #[test]
    fn delete_word_removes_current_word_only() {
        let mut s = state("foo bar baz");
        for (i, c) in "foo bar ba".chars().enumerate() {
            s.type_char(c, i as u128);
        }
        // cursor is inside "ba" (third word)
        s.delete_word();
        // back to just after "foo bar "
        assert_eq!(s.cursor(), 8);
        assert_eq!(s.typed().iter().collect::<String>(), "foo bar ");
    }

    #[test]
    fn delete_word_from_trailing_space() {
        let mut s = state("foo bar");
        for (i, c) in "foo ".chars().enumerate() {
            s.type_char(c, i as u128);
        }
        s.delete_word();
        assert_eq!(s.typed().iter().collect::<String>(), "");
    }

    #[test]
    fn words_mode_finishes_when_complete() {
        let mut s = TestState::new("hi".chars().collect(), TestKind::Words(1));
        s.type_char('h', 0);
        assert_eq!(s.status(), Status::Running);
        s.type_char('i', 10);
        assert_eq!(s.status(), Status::Finished);
        // further input ignored
        s.type_char('x', 20);
        assert_eq!(s.cursor(), 2);
    }

    #[test]
    fn overflow_is_clamped() {
        let mut s = state("hi");
        s.type_char('h', 0);
        s.type_char('i', 10);
        s.type_char('x', 20); // beyond target
        assert_eq!(s.cursor(), 2);
    }

    #[test]
    fn restart_same_keeps_target() {
        let mut s = state("cat");
        s.type_char('c', 0);
        s.restart_same();
        assert_eq!(s.cursor(), 0);
        assert_eq!(s.status(), Status::Idle);
        assert_eq!(s.target().iter().collect::<String>(), "cat");
        assert!(s.keys().is_empty());
    }
}
