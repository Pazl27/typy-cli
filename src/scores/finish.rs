use crate::scores::graph;
use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyEvent};
use crossterm::style::SetForegroundColor;
use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;

use crate::utils;

pub fn show_stats(
    mut stdout: &std::io::Stdout,
    lps_raw: Vec<i32>,
    words_raw: i32,
    incorrect_words: i32,
) -> Option<()> {
    let total_seconds = lps_raw.len() as i32;
    let minutes = total_seconds as f64 / 60.0;
    let wpm = (words_raw - incorrect_words) as f64 / minutes;
    let raw_wpm = words_raw as f64 / minutes;
    let accuracy = (1.0 - (incorrect_words as f64 / words_raw as f64)) * 100.0;

    stdout.execute(Clear(ClearType::All)).unwrap();

    // Draw infos
    stdout.execute(MoveTo(15, 16)).unwrap();
    stdout
        .execute(SetForegroundColor(crossterm::style::Color::Grey))
        .unwrap();
    print!("WPM");
    stdout.execute(MoveTo(15, 17)).unwrap();
    stdout
        .execute(SetForegroundColor(crossterm::style::Color::Yellow))
        .unwrap();
    print!("{:02}", wpm as i32);
    stdout.execute(MoveTo(15, 20)).unwrap();
    stdout
        .execute(SetForegroundColor(crossterm::style::Color::Grey))
        .unwrap();
    print!("RAW");
    stdout.execute(MoveTo(15, 21)).unwrap();
    stdout
        .execute(SetForegroundColor(crossterm::style::Color::Yellow))
        .unwrap();
    print!("{:02}", raw_wpm as i32);
    stdout.execute(MoveTo(37, 25)).unwrap();
    stdout
        .execute(SetForegroundColor(crossterm::style::Color::Yellow))
        .unwrap();
    print!("ACCURACY: {:.2}%", accuracy);

    graph::draw_graph(lps_raw).unwrap();

    loop {
        if let Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) = read()
        {
            if utils::close_typy(&code, &modifiers).is_some() {
                break None;
            }
        }
    }
}
