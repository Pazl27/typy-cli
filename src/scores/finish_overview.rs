use crate::scores::graph;
use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyEvent};
use crossterm::style::SetForegroundColor;
use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;
use anyhow::{Result, Context};

use crate::terminal;
use crate::scores::Stats;
use crate::config::theme::ThemeColors;

pub fn show_stats(
    mut stdout: &std::io::Stdout,
    stats: Stats,
    theme: &ThemeColors
) -> Result<()> {

    stdout.execute(Clear(ClearType::All)).context("Failed to clear terminal")?;

    // Draw infos
    stdout.execute(MoveTo(15, 16)).context("Failed to move cursor")?;
    stdout
        .execute(SetForegroundColor(theme.missing))
        .context("Failed to set foreground color")?;
    print!("WPM");
    stdout.execute(MoveTo(15, 17)).context("Failed to move cursor")?;
    stdout
        .execute(SetForegroundColor(theme.accent))
        .context("Failed to set foreground color")?;
    print!("{:02}", stats.wpm() as i32);
    stdout.execute(MoveTo(15, 20)).context("Failed to move cursor")?;
    stdout
        .execute(SetForegroundColor(theme.missing))
        .context("Failed to set foreground color")?;
    print!("RAW");
    stdout.execute(MoveTo(15, 21)).context("Failed to move cursor")?;
    stdout
        .execute(SetForegroundColor(theme.accent))
        .context("Failed to set foreground color")?;
    print!("{:02}", stats.raw_wpm() as i32);
    stdout.execute(MoveTo(37, 25)).context("Failed to move cursor")?;
    stdout
        .execute(SetForegroundColor(theme.accent))
        .context("Failed to set foreground color")?;
    print!("ACCURACY: {:.2}%", stats.accuracy());

    graph::draw_graph(stats.lps).context("Failed to draw graph")?;

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
