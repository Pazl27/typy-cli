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

fn main() {
    let mut stdout = stdout();

    let word_list = "Hello World I love you";
    let words = word_list.split_whitespace().collect::<Vec<&str>>();
    let mut position = 0;
    let mut jump_position = 0;
    let mut word_selected = 0;

    enable_raw_mode().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    let (cols, rows) = crossterm::terminal::size().unwrap();
    let x = cols / 2 - (word_list.chars().count() / 2) as u16;
    let y = rows / 2;

    stdout.execute(MoveTo(x, y)).unwrap();
    stdout.execute(SetForegroundColor(Color::Grey)).unwrap();
    words.iter().for_each(|word| {
        print!("{} ", word);
    });
    stdout.flush().unwrap();
    stdout.execute(MoveTo(x, y)).unwrap();

    loop {
        if let Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) = read()
        {
            match code {
                KeyCode::Esc => {
                    break;
                }
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    break;
                }
                KeyCode::Char(c) => {
                    if c == ' ' {
                        if word_selected == words.len() - 1 {
                            break;
                        }
                        if jump_position + 1 == position && jump_position != 0 {
                            continue;
                        }
                        jump_position = words
                            .iter()
                            .take(word_selected + 1)
                            .map(|word| word.chars().count() + 1)
                            .sum::<usize>()
                            - 1;
                        position = jump_position;
                        stdout.execute(MoveTo(x + position as u16, y)).unwrap();
                        word_selected += 1;
                    }
                    if c == word_list.chars().nth(position).unwrap() {
                        stdout.execute(SetForegroundColor(Color::White)).unwrap();
                        stdout.execute(MoveTo(x + position as u16, y)).unwrap();
                        print!("{}", word_list.chars().nth(position).unwrap());
                    } else {
                        stdout.execute(SetForegroundColor(Color::Red)).unwrap();
                        stdout.execute(MoveTo(x + position as u16, y)).unwrap();
                        print!("{}", word_list.chars().nth(position).unwrap());
                    }
                    if word_list.chars().nth(position).unwrap() == ' ' && c != ' ' {
                        word_selected += 1;
                    }
                    stdout.flush().unwrap();
                    position += 1;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();
    stdout.execute(ResetColor).unwrap(); 
    stdout.execute(Clear(ClearType::All)).unwrap(); 
    stdout.execute(MoveTo(0, 0)).unwrap(); 
    stdout.flush().unwrap();
}
