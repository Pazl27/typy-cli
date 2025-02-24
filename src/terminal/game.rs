use crate::error::{Error, Result};
use crossterm::cursor::{self, SetCursorStyle};
use crossterm::event::poll;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyEvent},
    style::{ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use super::keyboard::{handle_input, InputAction};
use std::io::stdout;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::cursor_style::CursorKind;
use crate::config::theme::ThemeColors;
use crate::mode::Mode;
use crate::scores::finish_overview;
use crate::scores::progress::{Data, Score};
use crate::scores::Stats;
use crate::word_provider;

pub struct Player {
    pub position_x: i32,
    pub position_y: i32,
}

impl Player {
    fn new() -> Self {
        Player {
            position_x: 0,
            position_y: 0,
        }
    }
}

pub struct Game {
    pub list: Vec<Vec<String>>,
    pub player: Player,
    pub jump_position: i32,
    pub selected_word_index: i32,
    quit: bool,
}

impl Game {
    fn new(list: Vec<Vec<String>>) -> Self {
        Game {
            list,
            player: Player::new(),
            jump_position: 0,
            selected_word_index: 0,
            quit: false,
        }
    }

    pub fn get_word_string(&self, index: i32) -> String {
        self.list.get(index as usize).unwrap().join(" ")
    }
}

pub fn run(mode: Mode, theme: ThemeColors) -> Result<()> {
    let mut stdout = stdout();

    let mut game = Game::new(
        word_provider::get_words(".local/share/typy/words.txt")
            .map_err(|e| Error::custom(format!("Failed to get words from file: {}", e)))?,
    );

    mode.transform(&mut game.list);

    let mut stats = Stats::new();

    setup_terminal(&stdout).map_err(|e| Error::custom(format!("Failed to setup terminal: {}", e)))?;

    let (x, y) = super::calc_middle_for_text().map_err(|e| Error::custom(format!("Failed to calculate terminal size: {}", e)))?;

    for i in 0..game.list.len() {
        print_words(
            x,
            y + i as u16,
            &game
                .list
                .get(i)
                .ok_or_else(|| Error::custom("Failed to get word from list"))?
                .to_vec(),
            &stdout,
            &theme,
        )?;
        stdout
            .execute(MoveTo(x, y as u16))
            .map_err(|e| Error::custom(format!("Failed to move cursor: {}", e)))?;
    }

    let timer_expired = Arc::new(AtomicBool::new(false));
    let timer_expired_clone = Arc::clone(&timer_expired);
    let remaining_time = Arc::new(Mutex::new(mode.duration));
    let remaining_time_clone = Arc::clone(&remaining_time);
    let mut remaining_prev: u64 = 0;

    let (tx, _) = mpsc::channel();

    let timer_thread = thread::spawn(move || {
        if let Err(e) = start_timer(mode.duration, timer_expired_clone, remaining_time_clone) {
            tx.send(e).expect("Failed to send error from timer thread");
        }
    });

    loop {
        if game.player.position_y == game.list.len() as i32 {
            break;
        }

        stdout
            .execute(MoveTo(
                x + game.player.position_x as u16,
                y + game.player.position_y as u16,
            ))
            .map_err(|e| Error::custom(format!("Failed to move cursor: {}", e)))?;

        if timer_expired.load(Ordering::Relaxed) {
            break;
        }

        {
            let remaining = *remaining_time
                .lock()
                .map_err(|e| Error::custom(format!("Failed to lock remaining time: {}", e)))?;
            stdout
                .execute(MoveTo(x, y - 2))
                .map_err(|e| Error::custom(format!("Failed to move cursor: {}", e)))?;
            stdout
                .execute(SetForegroundColor(theme.accent))
                .map_err(|e| Error::custom(format!("Failed to set foreground color: {}", e)))?;
            print!("{:02}", remaining);
            stdout.flush().map_err(|e| Error::custom(format!("Failed to flush stdout: {}", e)))?;
            stdout
                .execute(MoveTo(
                    x + game.player.position_x as u16,
                    y + game.player.position_y as u16,
                ))
                .map_err(|e| Error::custom(format!("Failed to move cursor: {}", e)))?;
            if remaining != remaining_prev {
                stats.add_letters();
            }
            remaining_prev = remaining;
        }

        if poll(Duration::from_millis(5)).map_err(|e| Error::custom(format!("Failed to poll for events: {}", e)))? {
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = read().map_err(|e| Error::custom(format!("Failed to read event: {}", e)))
            {
                if let Some(()) = super::close_typy(&code, &modifiers) {
                    timer_expired.store(true, Ordering::Relaxed);
                    game.quit = true;
                    break;
                }
                match handle_input(&mut game, &mut stdout, code, &mut stats, &theme, x, y)? {
                    InputAction::Continue => continue,
                    InputAction::Break => break,
                    InputAction::None => {}
                }
            }
        }
    }

    if !game.quit {
        stdout.execute(cursor::Hide)?;
        let score = Score::new(
            stats.wpm() as u32,
            stats.raw_wpm() as u32,
            stats.accuracy() as f32,
        );
        Data::save_data(score).map_err(|e| Error::custom(format!("Failed to save data: {}", e)))?;
        finish_overview::show_stats(&stdout, stats, &theme).map_err(|e| Error::custom(format!("Failed to show stats: {}", e)))?;
    }

    reset_terminal(&stdout).map_err(|e| Error::custom(format!("Failed to reset terminal: {}", e)))?;
    timer_expired.store(true, Ordering::Relaxed);
    timer_thread
        .join()
        .map_err(|e| Error::custom(format!("Failed to join timer thread: {:?}", e)))?;
    Ok(())
}

fn setup_terminal(mut stdout: &std::io::Stdout) -> Result<()> {
    let cursor_kind = CursorKind::new();

    enable_raw_mode()?;
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor_kind.style)?;

    Ok(())
}

fn reset_terminal(mut stdout: &std::io::Stdout) -> Result<()> {
    disable_raw_mode()?;
    stdout.execute(cursor::Show)?;
    stdout.execute(ResetColor)?;
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(MoveTo(0, 0))?;
    stdout.execute(SetCursorStyle::DefaultUserShape)?;
    stdout.flush()?;

    Ok(())
}

fn print_words(
    x: u16,
    y: u16,
    words: &Vec<String>,
    mut stdout: &std::io::Stdout,
    theme: &ThemeColors,
) -> Result<()> {
    stdout
        .execute(MoveTo(x, y))
        .map_err(|e| Error::custom(format!("Failed to move cursor: {}", e)))?;
    stdout
        .execute(SetForegroundColor(theme.missing))
        .map_err(|e| Error::custom(format!("Failed to set foreground color: {}", e)))?;
    words.iter().for_each(|word| {
        print!("{} ", word);
    });

    Ok(())
}

fn start_timer(
    duration: u64,
    timer_expired: Arc<AtomicBool>,
    remaining_time: Arc<Mutex<u64>>,
) -> Result<()> {
    let start = Instant::now();
    while start.elapsed().as_secs() < duration {
        if timer_expired.load(Ordering::Relaxed) {
            break;
        }
        let remaining = duration - start.elapsed().as_secs();
        {
            let mut remaining_time = remaining_time
                .lock()
                .map_err(|e| Error::custom(format!("Failed to lock remaining time: {}", e)))?;
            *remaining_time = remaining;
        }
        thread::sleep(Duration::from_secs(1));
    }
    timer_expired.store(true, Ordering::Relaxed);

    Ok(())
}
