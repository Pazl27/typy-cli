//! Home screen: ASCII art banner + test selectors + navigation to the other
//! screens. The selectors mirror the CLI flags (`-t`, `-w`, `--zen`, `-m`).

use anyhow::{Context, Result};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::time::Duration;

use crate::config::Settings;
use crate::mode::{Mode, ModeType};
use crate::terminal::ui::to_rat;
use crate::terminal::{Route, TestConfig, TestKind, Transition, Tui};

const BANNER: &[&str] = &[
    ",--------.,--.   ,--.,------.,--.   ,--.",
    "'--.  .--' \\  `.'  / |  .--. '\\  `.'  / ",
    "   |  |     '.    /  |  '--' | '.    /  ",
    "   |  |       |  |   |  | --'    |  |   ",
    "   `--'       `--'   `--'        `--'   ",
];

const TIME_PRESETS: [u64; 4] = [15, 30, 60, 120];
const WORDS_PRESETS: [usize; 4] = [10, 25, 50, 100];
const ROW_COUNT: usize = 8;
const ROW_START: usize = 7;

struct HomeState {
    sel: usize,
    ttype: usize, // 0 Time, 1 Words, 2 Zen
    time_idx: usize,
    words_idx: usize,
    uppercase: bool,
    punctuation: bool,
    numbers: bool,
    quotes: bool,
}

impl HomeState {
    fn new() -> Self {
        HomeState {
            sel: 0,
            ttype: 0,
            time_idx: 1, // 30s
            words_idx: 1, // 25 words
            uppercase: false,
            punctuation: false,
            numbers: false,
            quotes: false,
        }
    }

    fn change(&mut self, dir: i32) {
        match self.sel {
            0 => self.ttype = (self.ttype as i32 + dir).rem_euclid(3) as usize,
            1 => self.time_idx = (self.time_idx as i32 + dir).rem_euclid(4) as usize,
            2 => self.words_idx = (self.words_idx as i32 + dir).rem_euclid(4) as usize,
            3 => self.uppercase = !self.uppercase,
            4 => self.punctuation = !self.punctuation,
            5 => self.numbers = !self.numbers,
            6 => self.quotes = !self.quotes,
            _ => {}
        }
    }

    fn test_config(&self, settings: &Settings) -> TestConfig {
        let mut types = Vec::new();
        if self.uppercase {
            types.push(ModeType::Uppercase);
        }
        if self.punctuation {
            types.push(ModeType::Punctuation);
        }
        if self.numbers {
            types.push(ModeType::Numbers);
        }
        if self.quotes {
            types.push(ModeType::Quotes);
        }
        let mode = Mode::from_types(types, settings.mode_settings());

        let kind = match self.ttype {
            0 => TestKind::Time(TIME_PRESETS[self.time_idx]),
            1 => TestKind::Words(WORDS_PRESETS[self.words_idx]),
            _ => TestKind::Zen,
        };
        TestConfig { mode, kind }
    }

    fn rows(&self) -> Vec<(String, String)> {
        vec![
            (
                "Type".into(),
                ["Time", "Words", "Zen"][self.ttype].to_string(),
            ),
            ("Time".into(), format!("{}s", TIME_PRESETS[self.time_idx])),
            ("Words".into(), format!("{}", WORDS_PRESETS[self.words_idx])),
            ("Uppercase".into(), onoff(self.uppercase)),
            ("Punctuation".into(), onoff(self.punctuation)),
            ("Numbers".into(), onoff(self.numbers)),
            ("Quotes".into(), onoff(self.quotes)),
            ("▶  Start test".into(), String::new()),
        ]
    }
}

fn onoff(b: bool) -> String {
    if b {
        "on".into()
    } else {
        "off".into()
    }
}

pub fn run(term: &mut Tui, settings: &Settings) -> Result<Transition> {
    let mut st = HomeState::new();

    loop {
        term.draw(|f| render(f, &st, settings))
            .context("Failed to draw home")?;

        if poll(Duration::from_millis(150)).context("Failed to poll")? {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                ..
            }) = read().context("Failed to read event")?
            {
                if kind == crossterm::event::KeyEventKind::Release {
                    continue;
                }
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        st.sel = (st.sel + ROW_COUNT - 1) % ROW_COUNT
                    }
                    KeyCode::Down | KeyCode::Char('j') => st.sel = (st.sel + 1) % ROW_COUNT,
                    KeyCode::Left | KeyCode::Char('h') => st.change(-1),
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => st.change(1),
                    KeyCode::Enter => return Ok(Transition::StartTest(st.test_config(settings))),
                    KeyCode::Char('s') => return Ok(Transition::Goto(Route::Settings)),
                    KeyCode::Char('a') => return Ok(Transition::Goto(Route::Stats)),
                    KeyCode::Char('q') => return Ok(Transition::Quit),
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(Transition::Quit)
                    }
                    KeyCode::Esc => return Ok(Transition::Quit),
                    _ => {}
                }
            }
        }
    }
}

fn render(f: &mut ratatui::Frame, st: &HomeState, settings: &Settings) {
    let theme = settings.theme();
    let fg = to_rat(theme.fg);
    let accent = to_rat(theme.accent);
    let missing = to_rat(theme.missing);

    let area = f.area();
    let block = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(BANNER.len() as u16 + 1 + ROW_COUNT as u16 + 3),
        Constraint::Min(0),
    ])
    .split(area)[1];

    // Split the block into the centered content column and a full-width footer,
    // so the footer hint is never truncated by the narrow column.
    let block_rows = Layout::vertical([
        Constraint::Length(BANNER.len() as u16 + 1 + ROW_COUNT as u16),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(block);

    let cols = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(48),
        Constraint::Min(0),
    ])
    .split(block_rows[0])[1];

    let rows = Layout::vertical([
        Constraint::Length(BANNER.len() as u16),
        Constraint::Length(1),
        Constraint::Length(ROW_COUNT as u16),
    ])
    .split(cols);

    // Banner.
    let banner: Vec<Line> = BANNER
        .iter()
        .map(|l| Line::from(Span::styled(*l, Style::default().fg(accent))))
        .collect();
    f.render_widget(Paragraph::new(banner).alignment(Alignment::Center), rows[0]);

    // Option rows.
    let items: Vec<Line> = st
        .rows()
        .into_iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let selected = i == st.sel;
            let marker = if selected { "▸ " } else { "  " };
            let base = if selected { accent } else { fg };
            let line = if i == ROW_START {
                format!("{}{}", marker, label)
            } else {
                format!("{}{:<14}{}", marker, format!("{}:", label), value)
            };
            Line::from(Span::styled(
                line,
                Style::default()
                    .fg(base)
                    .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() }),
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(items).alignment(Alignment::Left), rows[2]);

    // Footer on two lines so the hints (incl. "a stats") fit narrow terminals.
    let footer = vec![
        Line::from("↑↓/jk move · ←→/hl change · enter start"),
        Line::from("s settings · a stats · q quit"),
    ];
    f.render_widget(
        Paragraph::new(footer)
            .style(Style::default().fg(missing))
            .alignment(Alignment::Center),
        block_rows[2],
    );
}
