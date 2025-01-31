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


    enable_raw_mode().unwrap();

    stdout.execute(Clear(ClearType::All)).unwrap();
    let (cols, rows) = crossterm::terminal::size().unwrap();
    let x = cols / 2 - 7;
    let y = rows / 2;

    let word_list = "Hello World";
    let words = word_list.split_whitespace().collect::<Vec<&str>>();
    let mut position = 0;
    let mut word_selected = 0;
    let mut right_letters = Vec::<char>::new();

    stdout.execute(MoveTo(x, y)).unwrap();
    stdout.execute(SetForegroundColor(Color::Grey)).unwrap();
    print!("{}", word_list);
    stdout.flush().unwrap();
    stdout.execute(MoveTo(x, y)).unwrap();

    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = read() {
            match code {
                KeyCode::Esc => {
                    break;
                }
                KeyCode::Char(c) => {
                    if c == ' ' {
                        if word_selected == words.len() {
                            break;
                        }
                        position = words.get(word_selected).unwrap().chars().count();
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
                    stdout.flush().unwrap();
                    position += 1;
                }
                _ => {}
            }
        }
    }

    stdout.execute(ResetColor).unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    stdout.flush().unwrap();
    disable_raw_mode().unwrap();
}

