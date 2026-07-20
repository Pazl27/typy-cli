use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph};
use ratatui::Frame;

use super::theme::UiTheme;
use crate::app::App;
use crate::config::graph_colors::Graph;
use crate::scores::Stats;

/// Overall size of the results panel. Kept small and centered rather than
/// filling the whole terminal.
const PANEL_WIDTH: u16 = 64;
const PANEL_HEIGHT: u16 = 18;
const GRAPH_HEIGHT: u16 = 10;

pub fn render(frame: &mut Frame, app: &App) {
    let theme = UiTheme::from(&app.theme);
    let Some(session) = app.session.as_ref() else {
        return;
    };
    let stats = &session.stats;

    let panel = centered_rect(frame.area(), PANEL_WIDTH, PANEL_HEIGHT);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),            // headline numbers
            Constraint::Length(1),            // spacer
            Constraint::Length(GRAPH_HEIGHT), // graph
            Constraint::Length(1),            // spacer
            Constraint::Length(1),            // footer
        ])
        .split(panel);

    render_headline(frame, rows[0], stats, &theme);
    render_graph(frame, rows[2], stats, &theme);
    render_footer(frame, rows[4], &theme);
}

/// A rect of the given size centered within `area` (clamped to it).
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

fn render_headline(frame: &mut Frame, area: Rect, stats: &Stats, theme: &UiTheme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    metric(frame, cols[0], "wpm", &safe(stats.wpm()).to_string(), theme);
    metric(
        frame,
        cols[1],
        "acc",
        &format!("{}%", safe(stats.accuracy())),
        theme,
    );
    metric(frame, cols[2], "raw", &safe(stats.raw_wpm()).to_string(), theme);
}

fn metric(frame: &mut Frame, area: Rect, label: &str, value: &str, theme: &UiTheme) {
    let text = vec![
        Line::from(Span::styled(
            label.to_string(),
            Style::default().fg(theme.missing),
        )),
        Line::from(Span::styled(
            value.to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
}

fn render_graph(frame: &mut Frame, area: Rect, stats: &Stats, theme: &UiTheme) {
    let colors = Graph::new();
    let data: Vec<(f64, f64)> = stats
        .lps
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    if data.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "not enough data for a graph",
                Style::default().fg(theme.missing),
            ))
            .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let x_max = (data.len().saturating_sub(1)).max(1) as f64;
    let y_max = stats.lps.iter().copied().max().unwrap_or(1).max(1) as f64;

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(colors.data.into()))
        .data(&data)];

    let chart = Chart::new(datasets)
        .block(Block::default())
        .x_axis(
            Axis::default()
                .title(Span::styled(
                    "seconds",
                    Style::default().fg(colors.title.into()),
                ))
                .style(Style::default().fg(colors.axis.into()))
                .bounds([0.0, x_max])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{}", x_max as u32)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled(
                    "letters",
                    Style::default().fg(colors.title.into()),
                ))
                .style(Style::default().fg(colors.axis.into()))
                .bounds([0.0, y_max])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{}", y_max as u32)),
                ]),
        );

    frame.render_widget(chart, area);
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &UiTheme) {
    let key = |k: &'static str| {
        Span::styled(
            k,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
    };
    let label = |t: &'static str| Span::styled(t, Style::default().fg(theme.missing));
    let line = Line::from(vec![
        key(" enter "),
        label("restart   "),
        key(" q "),
        label("home"),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

/// Round a metric to an integer, treating NaN/inf (empty test) as 0.
fn safe(value: f64) -> u32 {
    if value.is_finite() && value > 0.0 {
        value.round() as u32
    } else {
        0
    }
}
