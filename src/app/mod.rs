use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::config::theme::ThemeColors;
use crate::mode::Mode;
use crate::scores::progress::{Data, Score};
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
    pub mode: Mode,
    pub language: String,
    /// The active (or just-finished) typing test. Present on the Typing and
    /// Results screens.
    pub session: Option<TypingSession>,
}

impl App {
    pub fn new(mode: Mode, theme: ThemeColors, language: String) -> Self {
        App {
            screen: Screen::Home,
            should_quit: false,
            theme,
            mode,
            language,
            session: None,
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
        match TypingSession::new(&self.mode, &self.language) {
            Ok(session) => {
                self.session = Some(session);
                self.screen = Screen::Typing;
            }
            // If words can't be loaded there's nothing to type; stay home.
            Err(_) => self.screen = Screen::Home,
        }
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
            KeyCode::Char('s') => self.screen = Screen::Settings,
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
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.screen = Screen::Home;
        }
    }
}

/// Entry point: set up the terminal, run the render/event loop, tear down.
pub fn run(mode: Mode, theme: ThemeColors, language: String) -> Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new(mode, theme, language);

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
