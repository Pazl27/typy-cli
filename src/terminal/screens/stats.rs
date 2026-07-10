//! Advanced visual stats dashboard: summary tiles, an overall WPM-progression
//! chart across the whole history, a navigable table of every recorded test,
//! and the WPM curve of the selected test. Mistaken runs can be deleted.

use anyhow::{Context, Result};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use std::time::Duration;

use crate::config::Settings;
use crate::scores::graph;
use crate::scores::progress::{Averages, Data, Score};
use crate::terminal::ui::to_rat;
use crate::terminal::Tui;

pub fn run(term: &mut Tui, settings: &Settings) -> Result<()> {
    let mut scores = Data::get_scores().unwrap_or_default();
    Score::sort_scores(&mut scores);
    let mut averages = Data::get_averages().unwrap_or_default();
    let mut sel = 0usize;

    loop {
        if sel >= scores.len() && !scores.is_empty() {
            sel = scores.len() - 1;
        }

        term.draw(|f| render(f, settings, &scores, &averages, sel))
            .context("Failed to draw stats")?;

        if !poll(Duration::from_millis(150)).context("Failed to poll")? {
            continue;
        }
        let Event::Key(KeyEvent { code, kind, .. }) = read().context("Failed to read event")?
        else {
            continue;
        };
        if kind == KeyEventKind::Release {
            continue;
        }

        match code {
            KeyCode::Up | KeyCode::Char('k') => sel = sel.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                if sel + 1 < scores.len() {
                    sel += 1;
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(score) = scores.get(sel) {
                    Data::delete_score(score.timestamp).context("Failed to delete score")?;
                    scores = Data::get_scores().unwrap_or_default();
                    Score::sort_scores(&mut scores);
                    averages = Data::get_averages().unwrap_or_default();
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => break,
            _ => {}
        }
    }
    Ok(())
}

fn render(
    f: &mut ratatui::Frame,
    settings: &Settings,
    scores: &[Score],
    averages: &Averages,
    sel: usize,
) {
    let theme = settings.theme();
    let gcolors = settings.graph();
    let fg = to_rat(theme.fg);
    let accent = to_rat(theme.accent);
    let missing = to_rat(theme.missing);

    let area = f.area();
    let rows = Layout::vertical([
        Constraint::Length(4), // summary tiles (value + label inside a border)
        Constraint::Min(6),    // body
        Constraint::Length(1), // footer
    ])
    .split(area);

    render_tiles(f, rows[0], scores, averages, fg, accent, missing);

    if scores.is_empty() {
        f.render_widget(
            Paragraph::new("no tests recorded yet — go type something!")
                .style(Style::default().fg(missing))
                .alignment(Alignment::Center),
            rows[1],
        );
        footer(f, rows[2], missing);
        return;
    }

    // Body: progression chart on top; table + selected-test curve below.
    let body = Layout::vertical([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(rows[1]);

    // Overall progression: WPM per test, oldest -> newest.
    let progression: Vec<i32> = scores.iter().rev().map(|s| s.wpm as i32).collect();
    let prog_inner = titled_block(f, body[0], "progression — wpm per test", missing);
    graph::render_chart(f, prog_inner, &progression, &gcolors, "test", "wpm", true);

    let lower = Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(body[1]);

    render_table(f, lower[0], scores, sel, fg, accent, missing);

    let sel_inner = titled_block(f, lower[1], "selected test — wpm", missing);
    if let Some(s) = scores.get(sel) {
        if s.series.is_empty() {
            f.render_widget(
                Paragraph::new("no curve stored")
                    .style(Style::default().fg(missing))
                    .alignment(Alignment::Center),
                sel_inner,
            );
        } else {
            graph::render_chart(f, sel_inner, &s.series, &gcolors, "time in s", "wpm", false);
        }
    }

    footer(f, rows[2], missing);
}

/// Summary tiles: total tests, best & average WPM, best accuracy, avg consistency.
fn render_tiles(
    f: &mut ratatui::Frame,
    area: Rect,
    scores: &[Score],
    averages: &Averages,
    fg: Color,
    accent: Color,
    missing: Color,
) {
    let best_wpm = scores.iter().map(|s| s.wpm).max().unwrap_or(0);
    let best_acc = scores.iter().map(|s| s.accuracy).fold(0.0_f32, f32::max);

    let tiles = [
        ("tests", format!("{}", averages.total_tests())),
        ("best wpm", format!("{}", best_wpm)),
        ("avg wpm", format!("{:.0}", averages.wpm_avg.avg)),
        ("best acc", format!("{:.0}%", best_acc)),
        ("avg consistency", format!("{:.0}%", averages.consistency_avg.avg)),
    ];

    let cols = Layout::horizontal([Constraint::Ratio(1, tiles.len() as u32); 5]).split(area);
    for (i, (label, value)) in tiles.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(missing));
        let inner = block.inner(cols[i]);
        f.render_widget(block, cols[i]);
        let content = vec![
            Line::from(Span::styled(
                value.clone(),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(*label, Style::default().fg(fg))),
        ];
        f.render_widget(
            Paragraph::new(content).alignment(Alignment::Center),
            inner,
        );
    }
}

fn render_table(
    f: &mut ratatui::Frame,
    area: Rect,
    scores: &[Score],
    sel: usize,
    fg: Color,
    accent: Color,
    missing: Color,
) {
    let header = Row::new(
        ["date", "time", "wpm", "raw", "acc", "con", "mode"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(missing).add_modifier(Modifier::BOLD))),
    );
    let table_rows: Vec<Row> = scores
        .iter()
        .map(|s| {
            Row::new(vec![
                Cell::from(s.get_date()),
                Cell::from(s.get_time()),
                Cell::from(s.wpm.to_string()),
                Cell::from(s.raw.to_string()),
                Cell::from(format!("{:.0}%", s.accuracy)),
                Cell::from(format!("{:.0}%", s.consistency)),
                Cell::from(if s.mode.is_empty() {
                    "-".to_string()
                } else {
                    s.mode.clone()
                }),
            ])
        })
        .collect();
    let widths = [
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Min(6),
    ];
    let table = Table::new(table_rows, widths)
        .header(header)
        .style(Style::default().fg(fg))
        .row_highlight_style(Style::default().fg(accent).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");
    let mut ts = TableState::default();
    ts.select(Some(sel));
    f.render_stateful_widget(table, area, &mut ts);
}

/// Render a bordered, titled panel and return its inner drawing area.
fn titled_block(f: &mut ratatui::Frame, area: Rect, title: &str, color: Color) -> Rect {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(format!(" {} ", title), Style::default().fg(color)));
    let inner = block.inner(area);
    f.render_widget(block, area);
    inner
}

fn footer(f: &mut ratatui::Frame, area: Rect, color: Color) {
    f.render_widget(
        Paragraph::new("↑↓/jk select · d delete · esc back")
            .style(Style::default().fg(color))
            .alignment(Alignment::Center),
        area,
    );
}
