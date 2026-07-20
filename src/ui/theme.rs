use ratatui::style::Color;

use crate::config::theme::ThemeColors;

/// The theme palette converted to ratatui colors, ready for styling.
///
/// `ThemeColors` stores `crossterm` colors (from the config layer); ratatui
/// provides a `From` conversion for them, which this type funnels through so the
/// UI code never touches crossterm color types directly.
pub struct UiTheme {
    pub fg: Color,
    pub missing: Color,
    pub error: Color,
    pub accent: Color,
}

impl UiTheme {
    pub fn from(theme: &ThemeColors) -> Self {
        UiTheme {
            fg: theme.fg.into(),
            missing: theme.missing.into(),
            error: theme.error.into(),
            accent: theme.accent.into(),
        }
    }
}
