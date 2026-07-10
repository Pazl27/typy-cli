use anyhow::{Context, Result};
use crossterm::event::{poll, read, Event, KeyEventKind};
use crossterm::ExecutableCommand;
use std::io::stdout;
use std::time::{Duration, Instant};

use super::app::TestConfig;
use super::keybinds::{Action, KeybindPreset};
use super::keyboard::apply_typing;
use super::screens::finish;
use super::state::{Status, TestKind, TestState};
use super::ui;
use super::Tui;
use crate::config::Settings;
use crate::config::theme::ThemeColors;
use crate::mode::{Mode, ModeType};
use crate::scores::progress::{Data, Score};
use crate::scores::Stats;
use crate::word_provider;

/// Append more words when the cursor gets within this many chars of the end.
const TOPUP_THRESHOLD: usize = 40;
const TOPUP_WORDS: usize = 30;
/// How long WPM is suppressed at the very start (avoids absurd early values).
const WPM_WARMUP_MS: u128 = 500;
/// Tests shorter than this many typed words are not recorded (avoids polluting
/// stats with abandoned / accidental runs).
const MIN_WORDS_TO_SAVE: usize = 3;

/// Outcome of the interactive loop.
enum Outcome {
    Finished(u128),
    Quit,
}

/// Run tests inside an already set-up terminal. After each finished test the
/// result screen is shown; the restart gesture there loops into a fresh test.
pub fn run_test(term: &mut Tui, cfg: TestConfig, settings: &Settings) -> Result<()> {
    let TestConfig { mode, kind: base_kind } = cfg;
    let theme = settings.theme();
    let preset = settings.keybind_preset();
    let lang = settings.language();

    // Apply the configured cursor style for the typing caret.
    let _ = stdout().execute(settings.cursor_kind().style);

    loop {
        let target = gen_target(&mode, &lang, base_kind)?;
        // Quotes are a fixed sentence: run them as a word-count test.
        let kind = if mode.contains(ModeType::Quotes) {
            let words = target.split(|&c| c == ' ').filter(|w| !w.is_empty()).count();
            TestKind::Words(words.max(1))
        } else {
            base_kind
        };

        let mut state = TestState::new(target, kind);
        match game_loop(term, &mut state, &mode, &lang, kind, preset, &theme)? {
            Outcome::Finished(elapsed_ms) => {
                let stats = Stats::from_keys(state.keys(), elapsed_ms);
                if typed_word_count(state.typed()) >= MIN_WORDS_TO_SAVE {
                    let score = Score::new(
                        stats.wpm() as u32,
                        stats.raw_wpm() as u32,
                        stats.accuracy() as f32,
                        stats.consistency() as f32,
                        stats.wpm_series(),
                        mode.label(),
                    );
                    Data::save_data(score).context("Failed to save data")?;
                }
                // Restart gesture on the finish screen loops into a new test.
                if finish::show(term, &stats, settings).context("Failed to show finish screen")? {
                    continue;
                }
                break;
            }
            Outcome::Quit => break,
        }
    }

    Ok(())
}

fn typed_word_count(typed: &[char]) -> usize {
    typed.split(|&c| c == ' ').filter(|w| !w.is_empty()).count()
}

#[allow(clippy::too_many_arguments)]
fn game_loop(
    term: &mut Tui,
    state: &mut TestState,
    mode: &Mode,
    lang: &str,
    kind: TestKind,
    preset: KeybindPreset,
    theme: &ThemeColors,
) -> Result<Outcome> {
    let mut armed = false;
    let mut start: Option<Instant> = None;

    loop {
        let elapsed_ms = start.map(|s| s.elapsed().as_millis()).unwrap_or(0);

        if let TestKind::Time(d) = kind {
            if state.status() == Status::Running && elapsed_ms >= (d as u128) * 1000 {
                state.finish();
            }
        }
        if state.status() == Status::Finished {
            return Ok(Outcome::Finished(elapsed_ms));
        }

        if matches!(kind, TestKind::Time(_) | TestKind::Zen)
            && state.cursor() + TOPUP_THRESHOLD >= state.target().len()
        {
            let extra = gen_words(mode, lang, TOPUP_WORDS)?;
            let mut chars = vec![' '];
            chars.extend(extra.chars());
            state.extend_target(&chars);
        }

        let header = build_header(state, kind, elapsed_ms, &mode.label());
        let footer = build_footer(preset);
        term.draw(|f| ui::render(f, state, theme, &header, &footer))
            .context("Failed to draw frame")?;

        if poll(Duration::from_millis(100)).context("Failed to poll for events")? {
            match read().context("Failed to read event")? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    let (action, next_armed) = preset.map(key, armed);
                    armed = next_armed;
                    match action {
                        Action::Quit => return Ok(Outcome::Quit),
                        Action::NewTest => {
                            state.new_test(gen_target(mode, lang, kind)?);
                            start = None;
                        }
                        other => {
                            if start.is_none() && matches!(other, Action::Type(_)) {
                                start = Some(Instant::now());
                            }
                            let e = start.map(|s| s.elapsed().as_millis()).unwrap_or(0);
                            apply_typing(state, other, e);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn build_header(state: &TestState, kind: TestKind, elapsed_ms: u128, mode_label: &str) -> String {
    let live = Stats::from_keys(state.keys(), elapsed_ms.max(1));
    let wpm = if elapsed_ms < WPM_WARMUP_MS {
        0
    } else {
        live.wpm().round() as u32
    };

    let progress = match kind {
        TestKind::Time(d) => {
            let remaining = (d as u128).saturating_sub(elapsed_ms / 1000);
            format!("{}s", remaining)
        }
        TestKind::Words(w) => {
            let done = state.typed().iter().filter(|&&c| c == ' ').count();
            format!("{}/{}", done.min(w), w)
        }
        TestKind::Zen => "zen".to_string(),
    };

    format!("  {}   ·   {}   ·   {} wpm", mode_label, progress, wpm)
}

fn build_footer(preset: KeybindPreset) -> String {
    format!(
        "  esc quit  ·  ctrl+⌫ delete word  ·  restart: {}",
        preset.restart_hint()
    )
}

/// Generate `n` mode-transformed words joined by spaces.
fn gen_words(mode: &Mode, lang: &str, n: usize) -> Result<String> {
    let pool = word_provider::word_pool(lang, n).context("Failed to get words")?;
    let mut wrapped = vec![pool];
    mode.transform(&mut wrapped);
    Ok(wrapped.remove(0).join(" "))
}

/// Build the initial target text for a test kind.
fn gen_target(mode: &Mode, lang: &str, kind: TestKind) -> Result<Vec<char>> {
    if mode.contains(ModeType::Quotes) {
        return Ok(word_provider::random_quote(lang).chars().collect());
    }
    let n = match kind {
        TestKind::Words(w) => w.max(1),
        _ => word_provider::DEFAULT_POOL_WORDS,
    };
    Ok(gen_words(mode, lang, n)?.chars().collect())
}
