use crate::scores::graph;
use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyEvent};
use crossterm::style::SetForegroundColor;
use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;

use crate::utils;
use crate::scores::stats::Stats;
use crate::config::theme::ThemeColors;

pub fn show_stats(
    mut stdout: &std::io::Stdout,
    stats: Stats,
    theme: &ThemeColors
) -> Option<()> {

    let (_, y) = utils::calc_size();

    stdout.execute(Clear(ClearType::All)).unwrap();

    stdout.execute(MoveTo(15, (y / 2) + 13)).unwrap();
    stdout.execute(SetForegroundColor(theme.missing)).unwrap();
    print!("WPM:");

    stdout.execute(MoveTo(20, (y / 2) + 13)).unwrap();
    stdout.execute(SetForegroundColor(theme.accent)).unwrap();
    print!("{:02}", stats.wpm() as i32);

    stdout.execute(MoveTo(25, (y / 2) + 13)).unwrap();
    stdout.execute(SetForegroundColor(theme.missing)).unwrap();
    print!("RAW:");

    stdout.execute(MoveTo(30, (y / 2) + 13)).unwrap();
    stdout.execute(SetForegroundColor(theme.accent)).unwrap();
    print!("{:02}", stats.raw_wpm() as i32);

    stdout.execute(MoveTo(35, (y / 2) + 13)).unwrap();
    stdout.execute(SetForegroundColor(theme.missing)).unwrap();
    print!("ACCURACY:");

    stdout.execute(MoveTo(45, (y / 2) + 13)).unwrap();
    stdout.execute(SetForegroundColor(theme.accent)).unwrap();
    print!("{:.2}%", stats.accuracy());

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
