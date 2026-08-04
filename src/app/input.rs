use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{App, Game, Screen};

enum Post {
    None,
    Apply,
    Leave,
}

impl App {
    pub(super) fn handle_event(&mut self, event: Event) {
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
        if key.code == KeyCode::Esc {
            if self.direct {
                self.should_quit = true;
            } else {
                self.session = None;
                self.screen = Screen::Home;
            }
            return;
        }

        let Some(game) = self.session.as_mut() else {
            self.screen = Screen::Home;
            return;
        };
        match game {
            Game::Words(s) => match key.code {
                KeyCode::Backspace => s.backspace(),
                KeyCode::Char(' ') => s.space(),
                KeyCode::Char(c) => s.type_char(c),
                _ => {}
            },
            Game::Code(s) => match key.code {
                KeyCode::Backspace => s.backspace(),
                KeyCode::Enter => s.newline(),
                KeyCode::Char(c) => s.type_char(c),
                _ => {}
            },
        }
        if game.is_finished() {
            self.finish_test();
        }
    }

    fn handle_results_key(&mut self, key: KeyEvent) {
        if let Some(opened) = self.results_opened {
            if opened.elapsed() < Duration::from_millis(600) {
                return;
            }
        }

        if self.direct {
            self.should_quit = true;
            return;
        }
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
