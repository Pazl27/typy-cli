use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyEvent};
use crossterm::style::SetForegroundColor;
use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;

use crate::config::theme::ThemeColors;
use crate::error::{Error, Result};
use crate::scores::graph;
use crate::scores::Stats;
use crate::terminal;

pub fn show_stats(mut stdout: &std::io::Stdout, stats: Stats, theme: &ThemeColors) -> Result<()> {
    stdout.execute(Clear(ClearType::All)).map_err(Error::from)?;

    // Draw infos
    stdout.execute(MoveTo(15, 16)).map_err(Error::from)?;
    stdout
        .execute(SetForegroundColor(theme.missing))
        .map_err(Error::from)?;
    print!("WPM");
    stdout.execute(MoveTo(15, 17)).map_err(Error::from)?;
    stdout
        .execute(SetForegroundColor(theme.accent))
        .map_err(Error::from)?;
    print!("{:02}", stats.wpm() as i32);
    stdout.execute(MoveTo(15, 20)).map_err(Error::from)?;
    stdout
        .execute(SetForegroundColor(theme.missing))
        .map_err(Error::from)?;
    print!("RAW");
    stdout.execute(MoveTo(15, 21)).map_err(Error::from)?;
    stdout
        .execute(SetForegroundColor(theme.accent))
        .map_err(Error::from)?;
    print!("{:02}", stats.raw_wpm() as i32);
    stdout.execute(MoveTo(37, 25)).map_err(Error::from)?;
    stdout
        .execute(SetForegroundColor(theme.accent))
        .map_err(Error::from)?;
    print!("ACCURACY: {:.2}%", stats.accuracy());

    graph::draw_graph(stats.lps).map_err(Error::from)?;

    loop {
        if let Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) = read()
        {
            if terminal::close_typy(&code, &modifiers).is_some() {
                break;
            }
        }
    }

    Ok(())
}
