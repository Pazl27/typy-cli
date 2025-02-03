use crossterm::event::KeyModifiers;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent},
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use std::io::stdout;
use std::io::Write;

use crate::word_provider;

const LENGTH: i32 = 70;

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

pub fn run() {
    let mut stdout = stdout();

    let mut game = Game::new(word_provider::get_words());

    setup_terminal(&stdout);
    let (cols, rows) = crossterm::terminal::size().unwrap();
    let x = cols / 2 - (LENGTH / 2) as u16;
    let y = rows / 2 - 1;

    for i in 0..game.list.len() {
        print_words(x, y + i as u16, &game.list.get(i).unwrap(), &stdout);
        stdout.execute(MoveTo(x, y as u16)).unwrap();
    }

    let mut start_point = false;

    loop {
        if game.player.position_x
            == game.get_word_string(game.player.position_y).chars().count() as i32
        {
            start_point = true;
            game.player.position_x = 0;
            game.player.position_y += 1;
            game.jump_position = 0;
            game.selected_word_index = 0;
            if game.player.position_y == game.list.len() as i32 {
                break;
            }
        }
        if let Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) = read()
        {
            if let Some(()) = close_typy(&code, &modifiers) {
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

                        start_point = true;
                        game.player.position_x = 0;
                        game.player.position_y += 1;
                        game.jump_position = 0;
                        game.selected_word_index = 0;

                        stdout
                            .execute(MoveTo(
                                x + game.player.position_x as u16,
                                y + game.player.position_y as u16,
                            ))
                            .unwrap();
                        continue;
                    }
                    if game.jump_position + 1 == game.player.position_x && game.jump_position != 0 {
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
    reset_terminal(&stdout);
}

fn setup_terminal(mut stdout: &std::io::Stdout) {
    enable_raw_mode().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
}

fn reset_terminal(mut stdout: &std::io::Stdout) {
    disable_raw_mode().unwrap();
    stdout.execute(ResetColor).unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    stdout.execute(MoveTo(0, 0)).unwrap();
    stdout.flush().unwrap();
}

fn print_words(x: u16, y: u16, words: &Vec<String>, mut stdout: &std::io::Stdout) {
    stdout.execute(MoveTo(x, y)).unwrap();
    stdout.execute(SetForegroundColor(Color::Grey)).unwrap();
    words.iter().for_each(|word| {
        print!("{} ", word);
    });
}

fn close_typy(code: &KeyCode, modifiers: &KeyModifiers) -> Option<()> {
    match code {
        KeyCode::Esc => Some(()),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Some(()),
        _ => None,
    }
}
