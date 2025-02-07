use crate::scores::graph;
use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyEvent};
use crossterm::style::SetForegroundColor;
use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;

use crate::utils;
use crate::scores::stats::Stats;

pub fn show_stats(
    mut stdout: &std::io::Stdout,
    stats: Stats
) -> Option<()> {

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
    print!("{:02}", stats.wpm() as i32);
    stdout.execute(MoveTo(15, 20)).unwrap();
    stdout
        .execute(SetForegroundColor(crossterm::style::Color::Grey))
        .unwrap();
    print!("RAW");
    stdout.execute(MoveTo(15, 21)).unwrap();
    stdout
        .execute(SetForegroundColor(crossterm::style::Color::Yellow))
        .unwrap();
    print!("{:02}", stats.raw_wpm() as i32);
    stdout.execute(MoveTo(37, 25)).unwrap();
    stdout
        .execute(SetForegroundColor(crossterm::style::Color::Yellow))
        .unwrap();
    print!("ACCURACY: {:.2}%", stats.accuracy());

    graph::draw_graph(stats.lps).unwrap();

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
