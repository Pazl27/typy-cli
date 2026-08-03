use std::time::Instant;

use crate::persistence::config::save_settings;
use crate::domain::ModeType;
use crate::session::modifiers::Mode;
use crate::persistence::scores::progress::{Data, Score};
use crate::view::settings::SettingsState;
use crate::view::theme;
use crate::session::CodeSession;

use super::{App, Game, Screen, StatsData, TypingSession};

impl App {
    pub(super) fn start_test(&mut self) {
        let game = if self.code_mode {
            CodeSession::new(&self.code_language).map(Game::Code)
        } else {
            let mode = Mode::from_str(self.mode_tokens.iter().map(|s| s.as_str()).collect())
                .unwrap_or_else(|_| Mode::from_str(vec![ModeType::Normal.token()]).unwrap())
                .add_duration(self.time);
            TypingSession::new(&mode, &self.language).map(Game::Words)
        };

        match game {
            Ok(game) => {
                self.error = None;
                self.session = Some(game);
                self.screen = Screen::Typing;
            }
            Err(e) => {
                self.error = Some(format!("{e}"));
                self.screen = Screen::Home;
            }
        }
    }

    pub(super) fn open_stats(&mut self) {
        let mut scores = Data::get_scores().unwrap_or_default();
        Score::sort_scores(&mut scores);
        let averages = Data::get_averages().unwrap_or_else(|_| Data::default().averages);
        self.stats = Some(StatsData { averages, scores });
        self.screen = Screen::Stats;
    }

    pub(super) fn open_settings(&mut self) {
        self.settings = Some(SettingsState::new(
            &self.theme_name,
            &self.cursor_style,
            self.code_mode,
            &self.code_language,
            &self.language,
            &self.mode_tokens,
            self.time,
        ));
        self.screen = Screen::Settings;
    }

    pub(super) fn apply_settings(&mut self) {
        let Some((
            theme_name,
            cursor_style,
            code_mode,
            code_language,
            language,
            mode_tokens,
            time,
            mode_default,
        )) = self.settings.as_ref().map(|s| {
            (
                s.theme_name(),
                s.cursor_style(),
                s.code_mode(),
                s.code_language(),
                s.language(),
                s.mode_tokens(),
                s.time(),
                s.mode_default_string(),
            )
        }) else {
            return;
        };

        self.theme_name = theme_name;
        self.cursor_style = cursor_style;
        self.code_mode = code_mode;
        self.code_language = code_language;
        self.language = language;
        self.mode_tokens = mode_tokens;
        self.time = time;

        self.theme = theme::load(&self.theme_name);
        self.refresh_record();
        let _ = save_settings(
            &self.theme_name,
            &self.cursor_style,
            self.game_mode().content_token(),
            &self.code_language,
            &self.language,
            &mode_default,
            self.time,
        );
    }

    pub(super) fn finish_test(&mut self) {
        if let Some(game) = self.session.as_ref() {
            let stats = game.stats();
            let wpm = stats.wpm() as u32;
            let score = Score::new(wpm, stats.raw_wpm() as u32, stats.accuracy() as f32);

            if self.code_mode {
                self.new_record = false;
                let _ = Data::save_data(score, 0);
            } else {
                self.previous_record = Data::get_record(self.time).unwrap_or(0);
                self.new_record = wpm > self.previous_record;
                let _ = Data::save_data(score, self.time);
                self.refresh_record();
            }
        }
        self.screen = Screen::Results;
        self.results_opened = Some(Instant::now());
    }
}
