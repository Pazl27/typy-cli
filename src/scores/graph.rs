use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};
use ratatui::Frame;

use crate::config::graph_colors::Graph;

/// Render a line chart into `area` of the given frame. Reused by the finish
/// screen, the per-test detail, and the overall progression chart in the stats
/// screen.
pub fn render_chart(
    f: &mut Frame,
    area: Rect,
    data: &[i32],
    colors: &Graph,
    x_title: &str,
    y_title: &str,
    one_based: bool,
) {
    if data.is_empty() || area.width < 4 || area.height < 3 {
        return;
    }

    let points: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let max_y = data.iter().copied().max().unwrap_or(0).max(1);
    let x_end = (data.len().saturating_sub(1)).max(1) as f64;

    let (x_lo_label, x_hi_label) = x_axis_labels(data.len(), one_based);

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(colors.data))
        .data(&points)];

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .title(Span::styled(x_title.to_string(), Style::default().fg(colors.title)))
                .style(Style::default().fg(colors.axis))
                .bounds([0.0, x_end])
                .labels(vec![Span::from(x_lo_label), Span::from(x_hi_label)]),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled(y_title.to_string(), Style::default().fg(colors.title)))
                .style(Style::default().fg(colors.axis))
                .bounds([0.0, max_y as f64])
                .labels(vec![Span::from("0"), Span::from(max_y.to_string())]),
        );

    f.render_widget(chart, area);
}

/// X-axis end labels. Time axes are 0-based (sample 0 = 0s, so N samples end at
/// N-1); count axes ("test") read 1..N so N tests end at N.
fn x_axis_labels(len: usize, one_based: bool) -> (String, String) {
    if one_based {
        ("1".to_string(), len.max(1).to_string())
    } else {
        let x_end = len.saturating_sub(1).max(1);
        ("0".to_string(), x_end.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_axis_is_one_based() {
        // 4 tests -> axis reads 1 .. 4, not 0 .. 3
        assert_eq!(x_axis_labels(4, true), ("1".to_string(), "4".to_string()));
    }

    #[test]
    fn time_axis_is_zero_based() {
        // 31 samples (0..30s) -> axis reads 0 .. 30
        assert_eq!(x_axis_labels(31, false), ("0".to_string(), "30".to_string()));
    }
}
