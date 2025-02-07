use std::io;
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::symbols::{self};
use tui::text::Span;
use tui::widgets::Chart;
use tui::widgets::Dataset;
use tui::widgets::{Axis, GraphType};
use tui::Terminal;

pub fn draw_graph(data: Vec<i32>) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();

        let converted_data = data
            .iter()
            .enumerate()
            .map(|(i, &x)| (i as f64, x as f64))
            .collect::<Vec<(f64, f64)>>();

        let datasets = vec![Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&converted_data)];

        let end = data.len().to_string();
        let max_y = data.iter().max().unwrap().to_string();
        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .title(Span::styled("time in s", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 10.0])
                    .labels(
                        ["0", end.as_str()]
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
                        ["0", max_y.as_str()]
                            .iter()
                            .cloned()
                            .map(Span::from)
                            .collect(),
                    ),
            );
        let area = Rect::new(
            (size.width.saturating_sub(100)) / 2,
            (size.height.saturating_sub(10)) / 2,
            100,
            10,
        );

        f.render_widget(chart, area);
    })?;

    Ok(())
}
