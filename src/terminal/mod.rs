use crossterm::cursor::SetCursorStyle;
use crossterm::event::poll;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent},
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use std::io::stdout;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::scores::finish;
use crate::word_provider;
use crate::utils;

struct Player {
    position_x: i32,
    position_y: i32,
}

impl Player {
    fn new() -> Self {
        Player {
            position_x: 0,
            position_y: 0,
        }
    }
}

struct Game {
    list: Vec<Vec<String>>,
    player: Player,
    jump_position: i32,
    selected_word_index: i32,
}

impl Game {
    fn new(list: Vec<Vec<String>>) -> Self {
        Game {
            list,
            player: Player::new(),
            jump_position: 0,
            selected_word_index: 0,
        }
    }

    fn get_word_string(&self, index: i32) -> String {
        self.list.get(index as usize).unwrap().join(" ")
    }
}

pub fn run(timer_duration: u64) {
    let mut stdout = stdout();

    let mut game = Game::new(word_provider::get_words());

    setup_terminal(&stdout);

    let (x, y) = utils::calc_size();

    for i in 0..game.list.len() {
        print_words(x, y + i as u16, &game.list.get(i).unwrap(), &stdout);
        stdout.execute(MoveTo(x, y as u16)).unwrap();
    }

    let mut start_point = false;

    let timer_expired = Arc::new(AtomicBool::new(false));
    let timer_expired_clone = Arc::clone(&timer_expired);
    let remaining_time = Arc::new(Mutex::new(timer_duration));
    let remaining_time_clone = Arc::clone(&remaining_time);

    let timer_thread = thread::spawn(move || {
        start_timer(timer_duration, timer_expired_clone, remaining_time_clone);
    });

    loop {
        if timer_expired.load(Ordering::Relaxed) {
            break;
        }

        {
            let remaining = *remaining_time.lock().unwrap();
            stdout.execute(MoveTo(x, y - 2)).unwrap();
            stdout.execute(SetForegroundColor(Color::Yellow)).unwrap();
            print!("{:02}", remaining);
            stdout.flush().unwrap();
            stdout
                .execute(MoveTo(
                    x + game.player.position_x as u16,
                    y + game.player.position_y as u16,
                ))
                .unwrap();
        }

        if poll(Duration::from_millis(5)).unwrap() {
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = read()
            {
                if let Some(()) = utils::close_typy(&code, &modifiers) {
                    timer_expired.store(true, Ordering::Relaxed);
                    break;
                }
                if let KeyCode::Char(c) = code {
                    if c == ' ' {
                        if game.player.position_x == 0 {
                            continue;
                        }
                        if start_point {
                            start_point = false;
                            stdout
                                .execute(MoveTo(
                                    x + game.player.position_x as u16,
                                    y + game.player.position_y as u16,
                                ))
                                .unwrap();
                            continue;
                        }
                        if game.selected_word_index
                            == game
                                .list
                                .get(game.player.position_y as usize)
                                .unwrap()
                                .len() as i32
                                - 1
                        {
                            if game.player.position_y == game.list.len() as i32 {
                                break;
                            }

                            game.player.position_x = 0;
                            game.player.position_y += 1;
                            game.jump_position = 1;
                            game.selected_word_index = 0;

                            stdout
                                .execute(MoveTo(
                                    x + game.player.position_x as u16,
                                    y + game.player.position_y as u16,
                                ))
                                .unwrap();
                            continue;
                        }
                        if game.jump_position + 1 == game.player.position_x
                            && game.jump_position != 0
                        {
                            continue;
                        }
                        game.jump_position = game
                            .list
                            .get(game.player.position_y as usize)
                            .unwrap()
                            .iter()
                            .take(game.selected_word_index as usize + 1)
                            .map(|word| word.chars().count() + 1)
                            .sum::<usize>() as i32
                            - 1;
                        game.player.position_x = game.jump_position;
                        stdout
                            .execute(MoveTo(
                                x + game.player.position_x as u16,
                                y + game.player.position_y as u16,
                            ))
                            .unwrap();
                        game.selected_word_index += 1;
                    }
                    if c == game
                        .get_word_string(game.player.position_y)
                        .chars()
                        .nth(game.player.position_x as usize)
                        .unwrap()
                    {
                        stdout.execute(SetForegroundColor(Color::White)).unwrap();
                        stdout
                            .execute(MoveTo(
                                x + game.player.position_x as u16,
                                y + game.player.position_y as u16,
                            ))
                            .unwrap();
                        print!(
                            "{}",
                            game.get_word_string(game.player.position_y)
                                .chars()
                                .nth(game.player.position_x as usize)
                                .unwrap()
                        );
                    } else {
                        stdout.execute(SetForegroundColor(Color::Red)).unwrap();
                        stdout
                            .execute(MoveTo(
                                x + game.player.position_x as u16,
                                y + game.player.position_y as u16,
                            ))
                            .unwrap();
                        print!(
                            "{}",
                            game.get_word_string(game.player.position_y)
                                .chars()
                                .nth(game.player.position_x as usize)
                                .unwrap()
                        );
                    }
                    if game
                        .get_word_string(game.player.position_y)
                        .chars()
                        .nth(game.player.position_x as usize)
                        .unwrap()
                        == ' '
                        && c != ' '
                    {
                        game.selected_word_index += 1;
                    }
                    stdout.flush().unwrap();
                    game.player.position_x += 1;
                }
            }
        }
    }

    finish::show_stats(&stdout, Vec::new(), 0, 0);

    reset_terminal(&stdout);
    timer_thread.join().unwrap();
}

fn setup_terminal(mut stdout: &std::io::Stdout) {
    enable_raw_mode().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    stdout.execute(SetCursorStyle::SteadyBar).unwrap();
}

fn reset_terminal(mut stdout: &std::io::Stdout) {
    disable_raw_mode().unwrap();
    stdout.execute(ResetColor).unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    stdout.execute(MoveTo(0, 0)).unwrap();
    stdout.execute(SetCursorStyle::DefaultUserShape).unwrap();
    stdout.flush().unwrap();
}

fn print_words(x: u16, y: u16, words: &Vec<String>, mut stdout: &std::io::Stdout) {
    stdout.execute(MoveTo(x, y)).unwrap();
    stdout.execute(SetForegroundColor(Color::Grey)).unwrap();
    words.iter().for_each(|word| {
        print!("{} ", word);
    });
}

fn start_timer(duration: u64, timer_expired: Arc<AtomicBool>, remaining_time: Arc<Mutex<u64>>) {
    let start = Instant::now();
    while start.elapsed().as_secs() < duration {
        if timer_expired.load(Ordering::Relaxed) {
            break;
        }
        let remaining = duration - start.elapsed().as_secs();
        {
            let mut remaining_time = remaining_time.lock().unwrap();
            *remaining_time = remaining;
        }
        thread::sleep(Duration::from_secs(1));
    }
    timer_expired.store(true, Ordering::Relaxed);
}
