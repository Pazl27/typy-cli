use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::settings::SettingsState;
use crate::theme::Theme;

const PANEL_WIDTH: u16 = 52;
const VALUE_COL: usize = 16;

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let Some(state) = app.settings.as_ref() else {
        return;
    };

    let panel = render_panel(frame, state, theme);

    if state.open {
        render_popup(frame, panel, state, theme);
    }
}

fn render_panel(frame: &mut Frame, state: &SettingsState, theme: &Theme) -> Rect {
    let mut lines: Vec<Line> = Vec::new();
    for (i, row) in state.rows.iter().enumerate() {
        let active = i == state.cursor;
        let marker = if active { "> " } else { "  " };
        let value = &row.options[row.selected];

        let label_style = if active {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg)
        };

        let mut label = format!("{marker}{}", row.label);
        while label.chars().count() < VALUE_COL {
            label.push(' ');
        }

        lines.push(Line::from(vec![
            Span::styled(label, label_style),
            Span::styled(format!("{value} \u{25be}"), Style::default().fg(theme.fg)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(hint_line(theme));

    let height = lines.len() as u16 + 2;
    let panel = centered_rect(frame.area(), PANEL_WIDTH, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.missing))
        .title(Span::styled(
            " settings ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(Paragraph::new(lines).block(block), panel);
    panel
}

fn render_popup(frame: &mut Frame, panel: Rect, state: &SettingsState, theme: &Theme) {
    let row = &state.rows[state.cursor];

    let width = popup_width(row.options.iter().map(|s| s.as_str()), row.label);
    let max_visible = frame.area().height.saturating_sub(4).min(10);
    let height = (row.options.len() as u16 + 2).min(max_visible.max(3));

    let area = popup_rect(frame.area(), panel, state.cursor, width, height);

    let items: Vec<ListItem> = row
        .options
        .iter()
        .map(|opt| ListItem::new(opt.clone()))
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            format!(" {} ", row.label),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
        .highlight_symbol("");

    let mut list_state = ListState::default();
    list_state.select(Some(state.dropdown_cursor));

    frame.render_widget(Clear, area);
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn hint_line(theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(
        "j/k move   enter select   esc close",
        Style::default().fg(theme.missing),
    ))
}

fn popup_width<'a>(options: impl Iterator<Item = &'a str>, label: &str) -> u16 {
    let longest = options
        .map(|s| s.chars().count())
        .chain(std::iter::once(label.chars().count() + 2))
        .max()
        .unwrap_or(10);
    (longest as u16 + 4).clamp(12, 40)
}

fn popup_rect(screen: Rect, panel: Rect, row: usize, width: u16, height: u16) -> Rect {
    let width = width.min(screen.width);
    let height = height.min(screen.height);

    let anchor_x = panel.x + 1 + VALUE_COL as u16;
    let x = anchor_x.min((screen.x + screen.width).saturating_sub(width));

    let anchor_y = panel.y + 2 + row as u16;
    let y = anchor_y.min((screen.y + screen.height).saturating_sub(height));

    Rect {
        x,
        y,
        width,
        height,
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
