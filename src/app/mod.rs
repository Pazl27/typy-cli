use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::config::save_settings;
use crate::config::theme::ThemeColors;
use crate::mode::Mode;
use crate::scores::progress::{Data, Score};
use crate::settings::SettingsState;
use crate::tui::{events, Tui};
use crate::typing::TypingSession;
use crate::ui;

/// Which top-level view is currently on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Typing,
    Results,
    Settings,
}

/// Holds all state the UI renders from. The UI layer is a pure function of this
/// struct: `ui::render(frame, &app)`.
pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub theme: ThemeColors,
    /// Editable settings driving each test.
    pub language: String,
    pub mode_tokens: Vec<String>,
    pub time: u64,
    /// The active (or just-finished) typing test. Present on the Typing and
    /// Results screens.
    pub session: Option<TypingSession>,
    /// The settings screen's interactive state, present only while editing.
    pub settings: Option<SettingsState>,
}

impl App {
    pub fn new(theme: ThemeColors, language: String, mode_tokens: Vec<String>, time: u64) -> Self {
        App {
            screen: Screen::Home,
            should_quit: false,
            theme,
            language,
            mode_tokens,
            time,
            session: None,
            settings: None,
        }
    }

    /// Per-frame update independent of input, so the countdown keeps running and
    /// the test can end even while the user is idle.
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
            // If words can't be loaded there's nothing to type; stay home.
            Err(_) => self.screen = Screen::Home,
        }
    }

    fn open_settings(&mut self) {
        self.settings = Some(SettingsState::new(
            &self.language,
            &self.mode_tokens,
            self.time,
        ));
        self.screen = Screen::Settings;
    }

    /// Read the current settings selections into the live app state and persist
    /// them to the config file.
    fn apply_settings(&mut self) {
        let Some((language, mode_tokens, time, mode_default)) = self.settings.as_ref().map(|s| {
            (
                s.language(),
                s.mode_tokens(),
                s.time(),
                s.mode_default_string(),
            )
        }) else {
            return;
        };

        self.language = language;
        self.mode_tokens = mode_tokens;
        self.time = time;
        let _ = save_settings(&self.language, &mode_default, self.time);
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
    }

    /// Route an input event to the handler for the active screen.
    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            // Ignore key-release events (Windows sends both press and release).
            if key.kind != KeyEventKind::Press {
                return;
            }
            // Ctrl-C always quits, from any screen.
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.should_quit = true;
                return;
            }
            match self.screen {
                Screen::Home => self.handle_home_key(key),
                Screen::Typing => self.handle_typing_key(key),
                Screen::Results => self.handle_results_key(key),
                Screen::Settings => self.handle_settings_key(key),
            }
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('s') => self.open_settings(),
            _ => self.start_test(),
        }
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        let Some(session) = self.session.as_mut() else {
            self.screen = Screen::Home;
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.session = None;
                self.screen = Screen::Home;
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

    fn handle_results_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.start_test(),
            KeyCode::Char('q') | KeyCode::Esc => {
                self.session = None;
                self.screen = Screen::Home;
            }
            _ => {}
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        // Phase 1: mutate the settings state and decide what to do afterwards.
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

        // Phase 2: act now that the settings borrow has ended.
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

/// What `handle_settings_key` should do after releasing its borrow of the
/// settings state.
enum Post {
    None,
    Apply,
    Leave,
}

/// Entry point: set up the terminal, run the render/event loop, tear down.
pub fn run(
    theme: ThemeColors,
    language: String,
    mode_tokens: Vec<String>,
    time: u64,
) -> Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new(theme, language, mode_tokens, time);

    let result = (|| -> Result<()> {
        while !app.should_quit {
            app.tick();
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
