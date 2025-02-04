use tui::layout::Rect;
use tui::text::Span;
use std::io;
use tui::backend::CrosstermBackend;
use tui::style::{Color, Style};
use tui::symbols::{self};
use tui::widgets::{Axis, GraphType};
use tui::widgets::Chart;
use tui::widgets::Dataset;
use tui::widgets::Block;
use tui::Terminal;

pub fn draw_graph(data: Vec<i32>) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Draw the UI
    terminal.draw(|f| {
        let size = f.size();

        let datasets = vec![
            Dataset::default()
                .name("data1")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(Color::Cyan))
                .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Magenta))
                .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
        ];
        let end = data.len().to_string();
        let middle = (data.len() / 2).to_string();
        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .title(Span::styled("time", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 10.0])
                    .labels(
                        ["0", middle.as_str(), end.as_str()]
                            .iter()
                            .cloned()
                            .map(Span::from)
                            .collect(),
                    ),
            )
            .y_axis(
                Axis::default()
                    .title(Span::styled("letters", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 10.0])
                    .labels(
                        ["0.0", "5.0", "10.0"]
                            .iter()
                            .cloned()
                            .map(Span::from)
                            .collect(),
                    ),
            );
        f.render_widget(chart, size);
    })?;

    Ok(())
}
