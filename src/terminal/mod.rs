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

struct Game<'a> {
    list: Vec<&'a str>,
    player_position: i32,
    jump_position: i32,
    selected_word_index: i32,
}

impl<'a> Game<'a> {
    fn new(list: Vec<&'a str>) -> Self {
        Game {
            list,
            player_position: 0,
            jump_position: 0,
            selected_word_index: 0,
        }
    }

    fn get_word_string(&self) -> String {
        self.list.join(" ")
    }
}

pub fn run() {
    let mut stdout = stdout();

    let word_list = "Hello World I love you";
    word_provider::get_words();
    let mut game = Game::new(word_list.split_whitespace().collect::<Vec<&str>>());

    setup_terminal(&stdout);
    let (cols, rows) = crossterm::terminal::size().unwrap();
    let x = cols / 2 - (game.get_word_string().chars().count() / 2) as u16;
    let y = rows / 2;

    print_words(x, y, &game.list, &stdout);
    stdout.execute(MoveTo(x, y)).unwrap();

    loop {
        if game.player_position == game.get_word_string().chars().count() as i32 {
            break;
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
                    if game.selected_word_index == game.list.len() as i32 - 1 {
                        break;
                    }
                    if game.jump_position + 1 == game.player_position && game.jump_position != 0 {
                        continue;
                    }
                    game.jump_position = game
                        .list
                        .iter()
                        .take(game.selected_word_index as usize + 1)
                        .map(|word| word.chars().count() + 1)
                        .sum::<usize>() as i32
                        - 1;
                    game.player_position = game.jump_position;
                    stdout
                        .execute(MoveTo(x + game.player_position as u16, y))
                        .unwrap();
                    game.selected_word_index += 1;
                }
                if c == game
                    .get_word_string()
                    .chars()
                    .nth(game.player_position as usize)
                    .unwrap()
                {
                    stdout.execute(SetForegroundColor(Color::White)).unwrap();
                    stdout
                        .execute(MoveTo(x + game.player_position as u16, y))
                        .unwrap();
                    print!(
                        "{}",
                        game.get_word_string()
                            .chars()
                            .nth(game.player_position as usize)
                            .unwrap()
                    );
                } else {
                    stdout.execute(SetForegroundColor(Color::Red)).unwrap();
                    stdout
                        .execute(MoveTo(x + game.player_position as u16, y))
                        .unwrap();
                    print!(
                        "{}",
                        game.get_word_string()
                            .chars()
                            .nth(game.player_position as usize)
                            .unwrap()
                    );
                }
                if game
                    .get_word_string()
                    .chars()
                    .nth(game.player_position as usize)
                    .unwrap()
                    == ' '
                    && c != ' '
                {
                    game.selected_word_index += 1;
                }
                stdout.flush().unwrap();
                game.player_position += 1;
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

fn print_words(x: u16, y: u16, words: &Vec<&str>, mut stdout: &std::io::Stdout) {
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
