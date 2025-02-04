use crate::scores::graph;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::ExecutableCommand;
use std::io::Write;

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

    // clear the screen
    stdout.execute(crossterm::terminal::Clear(crossterm::terminal::ClearType::All)).unwrap();

    graph::draw_graph(lps_raw).unwrap();

    // Wait for 'q' key press to quit
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }) = event::read().unwrap()
        {
            return Some(());
        }
    }
}
