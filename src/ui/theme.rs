use ratatui::style::Color;

use crate::config::theme::ThemeColors;

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
