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
        let min_words = (mode.duration as usize).saturating_mul(5).max(60);
        let mut list = word_provider::get_words(language, min_words)
            .context("Failed to get words from file")?;
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
        word.typed.push(c);

        self.stats.letter_count += 1;

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
        let (correct, incorrect, extra, missed) = self.tally();
        let elapsed = self
            .start
            .map(|s| s.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        self.stats
            .finalize(correct, incorrect, extra, missed, self.cursor_word as i32, elapsed);
        self.finished = true;
    }

    fn tally(&self) -> (i32, i32, i32, i32) {
        let mut correct = 0;
        let mut incorrect = 0;
        let mut extra = 0;
        let mut missed = 0;

        let last = self.cursor_word.min(self.words.len().saturating_sub(1));
        for (wi, word) in self.words.iter().enumerate().take(last + 1) {
            let common = word.target.len().min(word.typed.len());
            for i in 0..common {
                if word.typed[i] == word.target[i] {
                    correct += 1;
                } else {
                    incorrect += 1;
                }
            }
            if word.typed.len() > word.target.len() {
                extra += (word.typed.len() - word.target.len()) as i32;
            }
            if wi < self.cursor_word && word.typed.len() < word.target.len() {
                missed += (word.target.len() - word.typed.len()) as i32;
            }
        }

        (correct, incorrect, extra, missed)
    }

    pub fn live_wpm(&self) -> u32 {
        let Some(start) = self.start else {
            return 0;
        };
        let minutes = start.elapsed().as_secs_f64() / 60.0;
        if minutes <= 0.0 {
            return 0;
        }
        let (correct, _, _, _) = self.tally();
        let chars = correct as f64 + self.cursor_word as f64;
        ((chars / 5.0) / minutes).max(0.0) as u32
    }
}
