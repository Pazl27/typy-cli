use std::time::Instant;

use anyhow::{Context, Result};

use crate::mode::Mode;
use crate::scores::Stats;
use crate::word_provider;

pub struct Word {
    pub target: Vec<char>,
    pub typed: Vec<char>,
}

impl Word {
    fn new(text: &str) -> Self {
        Word {
            target: text.chars().collect(),
            typed: Vec::new(),
        }
    }
}

pub struct TypingSession {
    pub words: Vec<Word>,
    pub cursor_word: usize,
    pub stats: Stats,
    pub duration: u64,
    start: Option<Instant>,
    sampled_secs: u64,
    finished: bool,
}

impl TypingSession {
    pub fn new(mode: &Mode, language: &str) -> Result<Self> {
        let mut list =
            word_provider::get_words(language).context("Failed to get words from file")?;
        mode.transform(&mut list);

        let words = list
            .into_iter()
            .flatten()
            .map(|w| Word::new(&w))
            .collect::<Vec<_>>();

        Ok(TypingSession {
            words,
            cursor_word: 0,
            stats: Stats::new(),
            duration: mode.duration,
            start: None,
            sampled_secs: 0,
            finished: false,
        })
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn remaining_secs(&self) -> u64 {
        match self.start {
            None => self.duration,
            Some(start) => self
                .duration
                .saturating_sub(start.elapsed().as_secs()),
        }
    }

    fn current(&mut self) -> Option<&mut Word> {
        self.words.get_mut(self.cursor_word)
    }

    pub fn type_char(&mut self, c: char) {
        if self.finished {
            return;
        }
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }

        let idx = self.cursor_word;
        let Some(word) = self.current() else {
            return;
        };
        let pos = word.typed.len();
        let correct = word.target.get(pos).is_some_and(|&t| t == c);
        word.typed.push(c);

        self.stats.letter_count += 1;
        if !correct {
            self.stats.incorrect_letters += 1;
        }

        if idx == self.words.len() - 1 {
            let word = &self.words[idx];
            if word.typed.len() >= word.target.len() {
                self.finish();
            }
        }
    }

    pub fn space(&mut self) {
        if self.finished {
            return;
        }
        if self.cursor_word + 1 >= self.words.len() {
            self.finish();
        } else {
            self.cursor_word += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.finished {
            return;
        }
        match self.current() {
            Some(word) if !word.typed.is_empty() => {
                word.typed.pop();
            }
            _ => {
                if self.cursor_word > 0 {
                    self.cursor_word -= 1;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        let Some(start) = self.start else {
            return;
        };
        if self.finished {
            return;
        }

        let elapsed = start.elapsed().as_secs();
        while self.sampled_secs < elapsed {
            self.stats.add_letters();
            self.sampled_secs += 1;
        }

        if elapsed >= self.duration {
            self.finish();
        }
    }

    fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.stats.add_letters();
        self.finished = true;
    }
}
