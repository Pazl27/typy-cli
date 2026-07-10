//! Post-test result screen, rendered with ratatui inside the shared terminal.

use anyhow::{Context, Result};
use crossterm::event::{poll, read, Event, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::time::Duration;

use crate::config::Settings;
use crate::scores::{graph, Stats};
use crate::terminal::ui::to_rat;
use crate::terminal::{Action, Tui};

fn display_char(c: char) -> String {
    match c {
        ' ' => "␣".to_string(),
        other => other.to_string(),
    }
}

/// Show the result screen. Returns `true` if the user asked to restart a new
/// test (via the keybind preset's restart gesture), `false` to go back.
pub fn show(term: &mut Tui, stats: &Stats, settings: &Settings) -> Result<bool> {
    let theme = settings.theme();
    let gcolors = settings.graph();
    let preset = settings.keybind_preset();
    let fg = to_rat(theme.fg);
    let accent = to_rat(theme.accent);
    let missing = to_rat(theme.missing);

    let series = stats.wpm_series();
    let errors = stats.top_errors(5);
    let mut armed = false;

    loop {
        term.draw(|f| {
            let area = f.area();
            let chunks = Layout::vertical([
                Constraint::Length(2), // metrics
                Constraint::Min(6),    // chart
                Constraint::Length(1), // errors
                Constraint::Length(1), // footer
            ])
            .split(area);

            let metrics = Line::from(vec![
                Span::styled("WPM ", Style::default().fg(missing)),
                Span::styled(
                    format!("{}   ", stats.wpm() as i32),
                    Style::default().fg(accent),
                ),
                Span::styled("RAW ", Style::default().fg(missing)),
                Span::styled(
                    format!("{}   ", stats.raw_wpm() as i32),
                    Style::default().fg(fg),
                ),
                Span::styled("ACC ", Style::default().fg(missing)),
                Span::styled(
                    format!("{:.1}%   ", stats.accuracy()),
                    Style::default().fg(fg),
                ),
                Span::styled("CONSISTENCY ", Style::default().fg(missing)),
                Span::styled(
                    format!("{:.1}%", stats.consistency()),
                    Style::default().fg(fg),
                ),
            ]);
            f.render_widget(
                Paragraph::new(metrics).alignment(Alignment::Center),
                chunks[0],
            );

            // Chart, centered with a horizontal margin.
            let chart_cols = Layout::horizontal([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(chunks[1]);
            graph::render_chart(f, chart_cols[1], &series, &gcolors, "time in s", "wpm", false);

            if !errors.is_empty() {
                let mut spans = vec![Span::styled("most missed:  ", Style::default().fg(missing))];
                for (c, errs, att) in &errors {
                    spans.push(Span::styled(
                        format!("{} {}/{}   ", display_char(*c), errs, att),
                        Style::default().fg(fg),
                    ));
                }
                f.render_widget(
                    Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
                    chunks[2],
                );
            }

            f.render_widget(
                Paragraph::new(format!(
                    "restart: {}  ·  esc to go back",
                    preset.restart_hint()
                ))
                .style(Style::default().fg(missing))
                .alignment(Alignment::Center),
                chunks[3],
            );
        })
        .context("Failed to draw finish screen")?;

        if poll(Duration::from_millis(200)).context("Failed to poll")? {
            if let Event::Key(key) = read().context("Failed to read event")? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                let (action, next_armed) = preset.map(key, armed);
                armed = next_armed;
                match action {
                    Action::NewTest => return Ok(true),
                    Action::Quit => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}
