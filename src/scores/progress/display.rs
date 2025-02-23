use std::io::{stdout, Write};
use std::time::Duration;

use crate::utils;

use super::*;
use anyhow::Result;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event, KeyEvent};
use crossterm::style::ResetColor;
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{cursor, ExecutableCommand};

const TABLE_WIDTH: u16 = 48;

pub fn draw() -> Result<()> {
    let mut scores = Score::get_scores()?;
    Score::sort_scores(&mut scores);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(TABLE_WIDTH)
        .set_header(vec![
            Cell::new("DATE").add_attribute(Attribute::Bold),
            Cell::new("TIME").add_attribute(Attribute::Bold),
            Cell::new("WPM").add_attribute(Attribute::Bold),
            Cell::new("RAW").add_attribute(Attribute::Bold),
            Cell::new("ACCURACY").add_attribute(Attribute::Bold),
        ]);

    for score in &scores {
        table.add_row(vec![
            Cell::new(score.get_date()),
            Cell::new(score.get_time()),
            Cell::new(score.wpm.to_string()),
            Cell::new(score.raw.to_string()),
            Cell::new(format!("{:.2}%", score.accuracy)),
        ]);
    }

    let (cols, rows) = terminal::size()?;
    let x = cols / 2 - (TABLE_WIDTH / 2);
    let y = rows / 2 - (scores.len() as u16 / 2);

    let mut stdout = stdout();
    setup_terminal(&mut stdout)?;
    stdout
        .execute(MoveTo(x, y))
        .context("Failed to move cursor")?;

    let table_string = table.to_string();
    let lines: Vec<&str> = table_string.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        stdout
            .execute(MoveTo(x, y + i as u16))
            .context("Failed to move cursor")?;
        write!(stdout, "{}", line)?;
    }
    stdout.flush()?;

    enable_raw_mode()?;
    loop {
        if poll(Duration::from_millis(5)).context("Failed to poll for events")? {
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = read().context("Failed to read event")
            {
                if let Some(()) = utils::close_typy(&code, &modifiers) {
                    break;
                }
            }
        }
    }

    reset_terminal(&mut stdout)?;

    Ok(())
}

fn setup_terminal(stdout: &mut std::io::Stdout) -> Result<()> {
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor::Hide)?;

    Ok(())
}

fn reset_terminal(stdout: &mut std::io::Stdout) -> Result<()> {
    disable_raw_mode()?;
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(MoveTo(0, 0))?;
    stdout.execute(ResetColor)?;
    stdout.execute(cursor::Show)?;
    stdout.flush()?;

    Ok(())
}
