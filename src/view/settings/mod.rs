mod constants;

use crate::content::code;
use crate::persistence::config::constants::defaults;
use crate::domain::{CursorStyle, ModeType};
use crate::view::theme::available_themes;
use crate::content::words::available_languages;

use constants::text;
use constants::values::TIME_OPTIONS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Mode,
    Theme,
    Cursor,
    CodeLanguage,
    Punctuation,
    Numbers,
    Uppercase,
    Language,
    Time,
}

pub struct Row {
    pub label: &'static str,
    pub field: Field,
    pub options: Vec<String>,
    pub selected: usize,
}

pub struct SettingsState {
    coding: bool,
    theme: String,
    cursor_style: String,
    code_language: String,
    punctuation: bool,
    numbers: bool,
    uppercase: bool,
    language: String,
    time: u64,

    themes: Vec<String>,
    cursors: Vec<String>,
    code_languages: Vec<String>,
    languages: Vec<String>,

    pub rows: Vec<Row>,
    pub cursor: usize,
    pub open: bool,
    pub dropdown_cursor: usize,
}

impl SettingsState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        theme: &str,
        cursor: &str,
        coding: bool,
        code_language: &str,
        language: &str,
        mode_tokens: &[String],
        time: u64,
    ) -> Self {
        let has = |mode: ModeType| mode_tokens.iter().any(|t| t == mode.token());

        let mut code_languages = vec![defaults::CODE_LANGUAGE.to_string()];
        code_languages.extend(code::available_languages());

        let code_language = if code_language.is_empty() {
            defaults::CODE_LANGUAGE.to_string()
        } else {
            code_language.to_string()
        };

        let mut state = SettingsState {
            coding,
            theme: theme.to_string(),
            cursor_style: cursor.to_string(),
            code_language,
            punctuation: has(ModeType::Punctuation),
            numbers: has(ModeType::Numbers),
            uppercase: has(ModeType::Uppercase),
            language: language.to_string(),
            time,
            themes: available_themes(),
            cursors: CursorStyle::ALL
                .iter()
                .map(|c| c.token().to_string())
                .collect(),
            code_languages,
            languages: available_languages(),
            rows: Vec::new(),
            cursor: 0,
            open: false,
            dropdown_cursor: 0,
        };
        state.rebuild_rows();
        state
    }

    fn rebuild_rows(&mut self) {
        let onoff = || vec![text::OPTION_OFF.to_string(), text::OPTION_ON.to_string()];
        let pick = |options: &[String], value: &str| {
            options.iter().position(|o| o == value).unwrap_or(0)
        };

        let mut rows = vec![
            Row {
                label: text::LABEL_MODE,
                field: Field::Mode,
                options: vec![
                    text::OPTION_TYPING.to_string(),
                    text::OPTION_CODING.to_string(),
                ],
                selected: self.coding as usize,
            },
            Row {
                label: text::LABEL_THEME,
                field: Field::Theme,
                options: self.themes.clone(),
                selected: pick(&self.themes, &self.theme),
            },
            Row {
                label: text::LABEL_CURSOR,
                field: Field::Cursor,
                options: self.cursors.clone(),
                selected: pick(&self.cursors, &self.cursor_style),
            },
        ];

        if self.coding {
            rows.push(Row {
                label: text::LABEL_CODE_LANGUAGE,
                field: Field::CodeLanguage,
                options: self.code_languages.clone(),
                selected: pick(&self.code_languages, &self.code_language),
            });
        } else {
            rows.push(Row {
                label: text::LABEL_PUNCTUATION,
                field: Field::Punctuation,
                options: onoff(),
                selected: self.punctuation as usize,
            });
            rows.push(Row {
                label: text::LABEL_NUMBERS,
                field: Field::Numbers,
                options: onoff(),
                selected: self.numbers as usize,
            });
            rows.push(Row {
                label: text::LABEL_UPPERCASE,
                field: Field::Uppercase,
                options: onoff(),
                selected: self.uppercase as usize,
            });
            rows.push(Row {
                label: text::LABEL_LANGUAGE,
                field: Field::Language,
                options: self.languages.clone(),
                selected: pick(&self.languages, &self.language),
            });
            rows.push(Row {
                label: text::LABEL_TIME,
                field: Field::Time,
                options: TIME_OPTIONS.iter().map(|t| t.to_string()).collect(),
                selected: TIME_OPTIONS.iter().position(|&t| t == self.time).unwrap_or(1),
            });
        }

        self.rows = rows;
        if self.cursor >= self.rows.len() {
            self.cursor = self.rows.len().saturating_sub(1);
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
        let field = self.rows[self.cursor].field;
        let idx = self.dropdown_cursor;
        match field {
            Field::Mode => self.coding = idx == 1,
            Field::Theme => self.theme = self.themes[idx].clone(),
            Field::Cursor => self.cursor_style = self.cursors[idx].clone(),
            Field::CodeLanguage => self.code_language = self.code_languages[idx].clone(),
            Field::Punctuation => self.punctuation = idx == 1,
            Field::Numbers => self.numbers = idx == 1,
            Field::Uppercase => self.uppercase = idx == 1,
            Field::Language => self.language = self.languages[idx].clone(),
            Field::Time => self.time = TIME_OPTIONS.get(idx).copied().unwrap_or(30),
        }
        self.open = false;
        self.rebuild_rows();
    }

    pub fn theme_name(&self) -> String {
        self.theme.clone()
    }

    pub fn cursor_style(&self) -> String {
        self.cursor_style.clone()
    }

    pub fn code_mode(&self) -> bool {
        self.coding
    }

    pub fn code_language(&self) -> String {
        self.code_language.clone()
    }

    pub fn language(&self) -> String {
        self.language.clone()
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn mode_tokens(&self) -> Vec<String> {
        let mut tokens = Vec::new();
        if self.uppercase {
            tokens.push(ModeType::Uppercase.token().to_string());
        }
        if self.punctuation {
            tokens.push(ModeType::Punctuation.token().to_string());
        }
        if self.numbers {
            tokens.push(ModeType::Numbers.token().to_string());
        }
        if tokens.is_empty() {
            tokens.push(ModeType::Normal.token().to_string());
        }
        tokens
    }

    pub fn mode_default_string(&self) -> String {
        self.mode_tokens().join(", ")
    }
}
