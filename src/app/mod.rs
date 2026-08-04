mod input;
mod update;

use std::io::stdout;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::execute;

use crate::domain::{CursorStyle, GameMode};
use crate::persistence::scores::progress::{Averages, Data, Score};
use crate::persistence::scores::Stats;
use crate::view::settings::SettingsState;
use crate::view::theme::Theme;
use crate::view::terminal::{events, Tui};
use crate::session::{CodeSession, TypingSession};
use crate::view::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Typing,
    Results,
    Settings,
    Stats,
}

pub struct StatsData {
    pub averages: Averages,
    pub scores: Vec<Score>,
}

pub enum Game {
    Words(TypingSession),
    Code(CodeSession),
}

impl Game {
    pub fn is_finished(&self) -> bool {
        match self {
            Game::Words(s) => s.is_finished(),
            Game::Code(s) => s.is_finished(),
        }
    }

    fn tick(&mut self) {
        match self {
            Game::Words(s) => s.tick(),
            Game::Code(s) => s.tick(),
        }
    }

    pub fn stats(&self) -> &Stats {
        match self {
            Game::Words(s) => &s.stats,
            Game::Code(s) => &s.stats,
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub theme: Theme,
    pub theme_name: String,
    pub cursor_style: String,
    pub language: String,
    pub mode_tokens: Vec<String>,
    pub time: u64,
    pub code_mode: bool,
    pub code_language: String,
    pub session: Option<Game>,
    pub settings: Option<SettingsState>,
    pub stats: Option<StatsData>,
    pub direct: bool,
    pub record: u32,
    pub previous_record: u32,
    pub new_record: bool,
    pub error: Option<String>,
    results_opened: Option<Instant>,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        theme: Theme,
        cursor_style: String,
        language: String,
        mode_tokens: Vec<String>,
        time: u64,
        code_mode: bool,
        code_language: String,
        direct: bool,
    ) -> Self {
        App {
            screen: Screen::Home,
            should_quit: false,
            theme_name: theme.name.clone(),
            theme,
            cursor_style,
            language,
            mode_tokens,
            time,
            code_mode,
            code_language,
            session: None,
            settings: None,
            stats: None,
            direct,
            record: Data::get_record(time).unwrap_or(0),
            previous_record: 0,
            new_record: false,
            error: None,
            results_opened: None,
        }
    }

    fn refresh_record(&mut self) {
        self.record = Data::get_record(self.time).unwrap_or(0);
    }

    fn game_mode(&self) -> GameMode {
        if self.code_mode {
            GameMode::Code
        } else {
            GameMode::Text
        }
    }

    fn tick(&mut self) {
        if self.screen != Screen::Typing {
            return;
        }
        if let Some(game) = self.session.as_mut() {
            game.tick();
            if game.is_finished() {
                self.finish_test();
            }
        }
    }

}

#[allow(clippy::too_many_arguments)]
pub fn run(
    theme: Theme,
    cursor_style: String,
    language: String,
    mode_tokens: Vec<String>,
    time: u64,
    code_mode: bool,
    code_language: String,
    direct: bool,
) -> Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new(
        theme,
        cursor_style,
        language,
        mode_tokens,
        time,
        code_mode,
        code_language,
        direct,
    );
    if direct {
        app.start_test();
    }

    let result = (|| -> Result<()> {
        while !app.should_quit {
            app.tick();
            if app.screen == Screen::Typing {
                let style = CursorStyle::from_token(&app.cursor_style)
                    .unwrap_or(CursorStyle::Block)
                    .to_set_cursor_style();
                let _ = execute!(stdout(), style);
            }
            tui.terminal.draw(|frame| ui::render(frame, &app))?;
            if let Some(event) = events::next(Duration::from_millis(100))? {
                app.handle_event(event);
            }
        }
        Ok(())
    })();

    tui.exit()?;
    result
}
