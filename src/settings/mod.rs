use std::collections::BTreeSet;

use crate::word_provider::available_languages;

const TIME_OPTIONS: &[u64] = &[15, 30, 60, 120];

fn mode_options() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("normal", vec!["normal"]),
        ("uppercase", vec!["uppercase"]),
        ("punctuation", vec!["punctuation"]),
        ("uppercase + punctuation", vec!["uppercase", "punctuation"]),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Language,
    Mode,
    Time,
}

pub struct Row {
    pub label: &'static str,
    pub field: Field,
    pub options: Vec<String>,
    pub selected: usize,
}

pub struct SettingsState {
    pub rows: Vec<Row>,
    pub cursor: usize,
    pub open: bool,
    pub dropdown_cursor: usize,
}

impl SettingsState {
    pub fn new(language: &str, mode_tokens: &[String], time: u64) -> Self {
        let languages = available_languages();
        let language_sel = languages.iter().position(|l| l == language).unwrap_or(0);

        let modes = mode_options();
        let mode_labels: Vec<String> = modes.iter().map(|(l, _)| l.to_string()).collect();
        let mode_sel = modes
            .iter()
            .position(|(_, toks)| tokens_match(toks, mode_tokens))
            .unwrap_or(0);

        let time_labels: Vec<String> = TIME_OPTIONS.iter().map(|t| t.to_string()).collect();
        let time_sel = TIME_OPTIONS.iter().position(|&t| t == time).unwrap_or(1);

        let rows = vec![
            Row {
                label: "language",
                field: Field::Language,
                options: languages,
                selected: language_sel,
            },
            Row {
                label: "mode",
                field: Field::Mode,
                options: mode_labels,
                selected: mode_sel,
            },
            Row {
                label: "time",
                field: Field::Time,
                options: time_labels,
                selected: time_sel,
            },
        ];

        SettingsState {
            rows,
            cursor: 0,
            open: false,
            dropdown_cursor: 0,
        }
    }

    pub fn move_down(&mut self) {
        if self.open {
            let len = self.rows[self.cursor].options.len();
            self.dropdown_cursor = (self.dropdown_cursor + 1).min(len.saturating_sub(1));
        } else {
            self.cursor = (self.cursor + 1).min(self.rows.len().saturating_sub(1));
        }
    }

    pub fn move_up(&mut self) {
        if self.open {
            self.dropdown_cursor = self.dropdown_cursor.saturating_sub(1);
        } else {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.dropdown_cursor = self.rows[self.cursor].selected;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn confirm(&mut self) {
        self.rows[self.cursor].selected = self.dropdown_cursor;
        self.open = false;
    }

    fn option_of(&self, field: Field) -> &str {
        let row = self
            .rows
            .iter()
            .find(|r| r.field == field)
            .expect("settings row missing");
        &row.options[row.selected]
    }

    pub fn language(&self) -> String {
        self.option_of(Field::Language).to_string()
    }

    pub fn time(&self) -> u64 {
        self.option_of(Field::Time).parse().unwrap_or(30)
    }

    pub fn mode_tokens(&self) -> Vec<String> {
        let label = self.option_of(Field::Mode);
        mode_options()
            .into_iter()
            .find(|(l, _)| *l == label)
            .map(|(_, toks)| toks.iter().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| vec!["normal".to_string()])
    }

    pub fn mode_default_string(&self) -> String {
        self.mode_tokens().join(", ")
    }
}

fn tokens_match(preset: &[&str], active: &[String]) -> bool {
    let a: BTreeSet<String> = preset.iter().map(|s| s.to_string()).collect();
    let b: BTreeSet<String> = active.iter().cloned().collect();
    a == b
}
