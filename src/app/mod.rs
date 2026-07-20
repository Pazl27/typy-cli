use std::io::stdout;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::cursor::SetCursorStyle;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;

use crate::config::save_settings;
use crate::mode::Mode;
use crate::scores::progress::{Averages, Data, Score};
use crate::settings::SettingsState;
use crate::theme::{self, Theme};
use crate::tui::{events, Tui};
use crate::typing::TypingSession;
use crate::ui;

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

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub theme: Theme,
    pub theme_name: String,
    pub cursor_style: String,
    pub language: String,
    pub mode_tokens: Vec<String>,
    pub time: u64,
    pub session: Option<TypingSession>,
    pub settings: Option<SettingsState>,
    pub stats: Option<StatsData>,
    pub direct: bool,
    results_opened: Option<Instant>,
}

impl App {
    pub fn new(
        theme: Theme,
        cursor_style: String,
        language: String,
        mode_tokens: Vec<String>,
        time: u64,
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
            session: None,
            settings: None,
            stats: None,
            direct,
            results_opened: None,
        }
    }

    fn tick(&mut self) {
        if self.screen != Screen::Typing {
            return;
        }
        if let Some(session) = self.session.as_mut() {
            session.tick();
            if session.is_finished() {
                self.finish_test();
            }
        }
    }

    fn start_test(&mut self) {
        let mode = Mode::from_str(self.mode_tokens.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(|_| Mode::from_str(vec!["normal"]).unwrap())
            .add_duration(self.time);

        match TypingSession::new(&mode, &self.language) {
            Ok(session) => {
                self.session = Some(session);
                self.screen = Screen::Typing;
            }
            Err(_) => self.screen = Screen::Home,
        }
    }

    fn open_stats(&mut self) {
        let mut scores = Data::get_scores().unwrap_or_default();
        Score::sort_scores(&mut scores);
        let averages = Data::get_averages().unwrap_or_else(|_| Data::default().averages);
        self.stats = Some(StatsData { averages, scores });
        self.screen = Screen::Stats;
    }

    fn open_settings(&mut self) {
        self.settings = Some(SettingsState::new(
            &self.theme_name,
            &self.cursor_style,
            &self.language,
            &self.mode_tokens,
            self.time,
        ));
        self.screen = Screen::Settings;
    }

    fn apply_settings(&mut self) {
        let Some((theme_name, cursor_style, language, mode_tokens, time, mode_default)) =
            self.settings.as_ref().map(|s| {
                (
                    s.theme_name(),
                    s.cursor_style(),
                    s.language(),
                    s.mode_tokens(),
                    s.time(),
                    s.mode_default_string(),
                )
            })
        else {
            return;
        };

        self.theme_name = theme_name;
        self.theme = theme::load(&self.theme_name);
        self.cursor_style = cursor_style;
        self.language = language;
        self.mode_tokens = mode_tokens;
        self.time = time;
        let _ = save_settings(
            &self.theme_name,
            &self.cursor_style,
            &self.language,
            &mode_default,
            self.time,
        );
    }

    fn finish_test(&mut self) {
        if let Some(session) = self.session.as_ref() {
            let score = Score::new(
                session.stats.wpm() as u32,
                session.stats.raw_wpm() as u32,
                session.stats.accuracy() as f32,
            );
            let _ = Data::save_data(score);
        }
        self.screen = Screen::Results;
        self.results_opened = Some(Instant::now());
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.should_quit = true;
                return;
            }
            match self.screen {
                Screen::Home => self.handle_home_key(key),
                Screen::Typing => self.handle_typing_key(key),
                Screen::Results => self.handle_results_key(key),
                Screen::Settings => self.handle_settings_key(key),
                Screen::Stats => self.handle_stats_key(key),
            }
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('s') => self.open_settings(),
            KeyCode::Char('p') => self.open_stats(),
            _ => self.start_test(),
        }
    }

    fn handle_stats_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p')) {
            self.stats = None;
            self.screen = Screen::Home;
        }
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        let Some(session) = self.session.as_mut() else {
            self.screen = Screen::Home;
            return;
        };
        match key.code {
            KeyCode::Esc => {
                if self.direct {
                    self.should_quit = true;
                } else {
                    self.session = None;
                    self.screen = Screen::Home;
                }
                return;
            }
            KeyCode::Backspace => session.backspace(),
            KeyCode::Char(' ') => session.space(),
            KeyCode::Char(c) => session.type_char(c),
            _ => {}
        }
        if session.is_finished() {
            self.finish_test();
        }
    }

    fn handle_results_key(&mut self, _key: KeyEvent) {
        if let Some(opened) = self.results_opened {
            if opened.elapsed() < Duration::from_millis(600) {
                return;
            }
        }

        if self.direct {
            self.should_quit = true;
            return;
        }
        match _key.code {
            KeyCode::Enter => self.start_test(),
            KeyCode::Char('q') | KeyCode::Esc => {
                self.session = None;
                self.screen = Screen::Home;
            }
            _ => {}
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        let post = {
            let Some(st) = self.settings.as_mut() else {
                self.screen = Screen::Home;
                return;
            };
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    st.move_down();
                    Post::None
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    st.move_up();
                    Post::None
                }
                KeyCode::Enter | KeyCode::Char('l') | KeyCode::Char(' ') => {
                    if st.open {
                        st.confirm();
                        Post::Apply
                    } else {
                        st.open();
                        Post::None
                    }
                }
                KeyCode::Char('h') => {
                    if st.open {
                        st.close();
                    }
                    Post::None
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    if st.open {
                        st.close();
                        Post::None
                    } else {
                        Post::Leave
                    }
                }
                _ => Post::None,
            }
        };

        match post {
            Post::Apply => self.apply_settings(),
            Post::Leave => {
                self.settings = None;
                self.screen = Screen::Home;
            }
            Post::None => {}
        }
    }
}

enum Post {
    None,
    Apply,
    Leave,
}

fn cursor_shape(name: &str) -> SetCursorStyle {
    let blink = name.contains("blink");
    if name.contains("underline") {
        if blink {
            SetCursorStyle::BlinkingUnderScore
        } else {
            SetCursorStyle::SteadyUnderScore
        }
    } else if name.contains("bar") {
        if blink {
            SetCursorStyle::BlinkingBar
        } else {
            SetCursorStyle::SteadyBar
        }
    } else if blink {
        SetCursorStyle::BlinkingBlock
    } else {
        SetCursorStyle::SteadyBlock
    }
}

pub fn run(
    theme: Theme,
    cursor_style: String,
    language: String,
    mode_tokens: Vec<String>,
    time: u64,
    direct: bool,
) -> Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new(theme, cursor_style, language, mode_tokens, time, direct);
    if direct {
        app.start_test();
    }

    let result = (|| -> Result<()> {
        while !app.should_quit {
            app.tick();
            if app.screen == Screen::Typing {
                let _ = execute!(stdout(), cursor_shape(&app.cursor_style));
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
