use std::time::Instant;

use anyhow::{Context, Result};

use crate::code_provider;
use crate::scores::Stats;

pub struct CodeSession {
    pub language: String,
    pub path: String,
    pub url: String,
    pub target: Vec<char>,
    pub typeable: Vec<bool>,
    pub typed: Vec<Option<char>>,
    pub cursor: usize,
    pub stats: Stats,
    start: Option<Instant>,
    sampled_secs: u64,
    finished: bool,
}

impl CodeSession {
    pub fn new(selector: &str) -> Result<Self> {
        let snippet = code_provider::get_snippet(selector).context("Failed to get code snippet")?;
        let target: Vec<char> = snippet.content.chars().collect();
        let typeable = compute_typeable(&target);
        let len = target.len();

        let mut session = CodeSession {
            language: snippet.language,
            path: snippet.path,
            url: snippet.url,
            target,
            typeable,
            typed: vec![None; len],
            cursor: 0,
            stats: Stats::new(),
            start: None,
            sampled_secs: 0,
            finished: false,
        };
        session.cursor = session.next_typeable(0);
        if session.cursor >= len {
            session.finished = true;
        }
        Ok(session)
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn next_typeable(&self, from: usize) -> usize {
        let mut i = from;
        while i < self.target.len() && !self.typeable[i] {
            i += 1;
        }
        i
    }

    pub fn type_char(&mut self, c: char) {
        if self.finished || self.cursor >= self.target.len() {
            return;
        }
        if self.target[self.cursor] == '\n' {
            return;
        }
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }
        self.typed[self.cursor] = Some(c);
        self.stats.letter_count += 1;
        self.advance();
    }

    pub fn newline(&mut self) {
        if self.finished || self.cursor >= self.target.len() {
            return;
        }
        if self.target[self.cursor] != '\n' {
            return;
        }
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }
        self.typed[self.cursor] = Some('\n');
        self.stats.letter_count += 1;
        self.advance();
    }

    fn advance(&mut self) {
        self.cursor = self.next_typeable(self.cursor + 1);
        if self.cursor >= self.target.len() {
            self.finish();
        }
    }

    pub fn backspace(&mut self) {
        if self.finished {
            return;
        }
        let mut i = self.cursor;
        while i > 0 {
            i -= 1;
            if self.typeable[i] {
                self.typed[i] = None;
                self.cursor = i;
                return;
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
    }

    fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.stats.add_letters();
        let (correct, incorrect, missed, newlines) = self.tally();
        let elapsed = self.start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
        self.stats
            .finalize(correct, incorrect, 0, missed, newlines, elapsed);
        self.finished = true;
    }

    fn tally(&self) -> (i32, i32, i32, i32) {
        let mut correct = 0;
        let mut incorrect = 0;
        let mut missed = 0;
        let mut newlines = 0;

        for i in 0..self.target.len() {
            if !self.typeable[i] {
                continue;
            }
            let target = self.target[i];
            match self.typed[i] {
                Some(_) if target == '\n' => newlines += 1,
                Some(c) if c == target => correct += 1,
                Some(_) => incorrect += 1,
                None if target != '\n' => missed += 1,
                None => {}
            }
        }

        (correct, incorrect, missed, newlines)
    }

    pub fn live_wpm(&self) -> u32 {
        let Some(start) = self.start else {
            return 0;
        };
        let minutes = start.elapsed().as_secs_f64() / 60.0;
        if minutes <= 0.0 {
            return 0;
        }
        let (correct, _, _, newlines) = self.tally();
        (((correct + newlines) as f64 / 5.0) / minutes).max(0.0) as u32
    }
}

fn compute_typeable(target: &[char]) -> Vec<bool> {
    let mut typeable = vec![true; target.len()];
    let mut at_line_start = true;
    for (i, &c) in target.iter().enumerate() {
        if c == '\n' {
            at_line_start = true;
            continue;
        }
        if at_line_start && c == ' ' {
            typeable[i] = false;
            continue;
        }
        at_line_start = false;
    }
    typeable
}
