use std::io::{stdout, Write};
use std::time::Duration;

use crate::terminal;

use super::*;
use anyhow::{Context, Result};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event, KeyEvent};
use crossterm::style::ResetColor;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType};
use crossterm::{cursor, ExecutableCommand};

const TABLE_WIDTH: u16 = 48;

pub fn draw() -> Result<()> {
    let mut stdout = stdout();
    setup_terminal(&mut stdout)?;

    let averages = draw_averages(&mut stdout)?;
    draw_progress(&mut stdout, averages)?;

    enable_raw_mode()?;
    loop {
        if poll(Duration::from_millis(5)).context("Failed to poll for events")? {
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = read().context("Failed to read event")
            {
                if let Some(()) = terminal::close_typy(&code, &modifiers) {
                    break;
                }
            }
        }
    }

    reset_terminal(&mut stdout)?;

    Ok(())
}

fn draw_averages(stdout: &mut std::io::Stdout) -> Result<Averages> {
    let averages = Data::get_averages()?;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(TABLE_WIDTH)
        .set_header(vec![
            Cell::new("Avg: WPM")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center),
            Cell::new("Avg: RAW")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center),
            Cell::new("Avg: ACCURACY")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new(format!("{:.2}", averages.wpm_avg.avg)).set_alignment(CellAlignment::Center),
            Cell::new(format!("{:.2}", averages.raw_avg.avg)).set_alignment(CellAlignment::Center),
            Cell::new(format!("{:.2}%", averages.accuracy_avg.avg))
                .set_alignment(CellAlignment::Center),
        ]);

    let (cols, _) = size()?;
    let x = cols / 2 - (39 / 2);
    let y = 8;

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

    Ok(averages)
}

fn draw_progress(stdout: &mut std::io::Stdout, averages: Averages) -> Result<()> {
    let mut scores = Data::get_scores()?;
    Score::sort_scores(&mut scores);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(TABLE_WIDTH)
        .set_header(vec![
            Cell::new("DATE")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center),
            Cell::new("TIME")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center),
            Cell::new("WPM")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center),
            Cell::new("RAW")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center),
            Cell::new("ACCURACY")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center),
        ]);

    for score in &scores {
        let wpm_color = if score.wpm >= averages.wpm_avg.avg as u32 {
            Color::Green
        } else {
            Color::Red
        };
        let raw_color = if score.raw >= averages.raw_avg.avg as u32 {
            Color::Green
        } else {
            Color::Red
        };
        let accuracy_color = if score.accuracy >= averages.accuracy_avg.avg {
            Color::Green
        } else {
            Color::Red
        };

        table.add_row(vec![
            Cell::new(score.get_date()).set_alignment(CellAlignment::Center),
            Cell::new(score.get_time()).set_alignment(CellAlignment::Center),
            Cell::new(score.wpm.to_string())
                .fg(wpm_color)
                .set_alignment(CellAlignment::Center),
            Cell::new(score.raw.to_string())
                .fg(raw_color)
                .set_alignment(CellAlignment::Center),
            Cell::new(format!("{:.2}%", score.accuracy))
                .fg(accuracy_color)
                .set_alignment(CellAlignment::Center),
        ]);
    }

    let (cols, rows) = size()?;
    let x = cols / 2 - (TABLE_WIDTH / 2);
    let y = rows / 2 - (scores.len() as u16 / 2);

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
