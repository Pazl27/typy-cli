use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::{App, StatsData};
use crate::scores::progress::Averages;
use crate::theme::Theme;

const PANEL_WIDTH: u16 = 60;

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let Some(data) = app.stats.as_ref() else {
        return;
    };

    let rows = data.scores.len() as u16;
    let height = (rows + 9).min(frame.area().height);
    let panel = centered_rect(frame.area(), PANEL_WIDTH, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.missing))
        .title(Span::styled(
            " stats ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    render_averages(frame, areas[0], &data.averages, theme);
    render_scores(frame, areas[2], data, theme);
    render_footer(frame, areas[3], theme);
}

fn render_averages(frame: &mut Frame, area: Rect, averages: &Averages, theme: &Theme) {
    let muted = Style::default().fg(theme.missing);
    let value = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);
    let line = Line::from(vec![
        Span::styled("avg  ", muted),
        Span::styled(format!("{:.1} wpm", averages.wpm_avg.avg), value),
        Span::styled("   ", muted),
        Span::styled(format!("{:.1} raw", averages.raw_avg.avg), value),
        Span::styled("   ", muted),
        Span::styled(format!("{:.1}% acc", averages.accuracy_avg.avg), value),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_scores(frame: &mut Frame, area: Rect, data: &StatsData, theme: &Theme) {
    if data.scores.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "no games played yet",
                Style::default().fg(theme.missing),
            ))
            .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let header_style = Style::default()
        .fg(theme.missing)
        .add_modifier(Modifier::BOLD);
    let header = Row::new(
        ["date", "time", "wpm", "raw", "acc"]
            .into_iter()
            .map(|h| Cell::from(Line::from(h).alignment(Alignment::Center))),
    )
    .style(header_style);

    let avg = &data.averages;
    let rows = data.scores.iter().map(|score| {
        let wpm_color = good_bad(score.wpm as f32, avg.wpm_avg.avg, theme);
        let raw_color = good_bad(score.raw as f32, avg.raw_avg.avg, theme);
        let acc_color = good_bad(score.accuracy, avg.accuracy_avg.avg, theme);

        Row::new(vec![
            centered(score.get_date(), theme.fg),
            centered(score.get_time(), theme.fg),
            centered(score.wpm.to_string(), wpm_color),
            centered(score.raw.to_string(), raw_color),
            centered(format!("{:.1}%", score.accuracy), acc_color),
        ])
    });

    let widths = [
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(9),
    ];

    frame.render_widget(Table::new(rows, widths).header(header), area);
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &Theme) {
    frame.render_widget(
        Paragraph::new(Span::styled(
            "esc  back",
            Style::default().fg(theme.missing),
        ))
        .alignment(Alignment::Center),
        area,
    );
}

fn centered(text: String, color: ratatui::style::Color) -> Cell<'static> {
    Cell::from(Line::from(text).alignment(Alignment::Center)).style(Style::default().fg(color))
}

fn good_bad(value: f32, average: f32, theme: &Theme) -> ratatui::style::Color {
    if value >= average {
        theme.graph_data
    } else {
        theme.error
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}
