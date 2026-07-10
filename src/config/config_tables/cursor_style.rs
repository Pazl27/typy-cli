use crossterm::cursor::SetCursorStyle;

pub struct CursorKind {
    pub style: SetCursorStyle,
}

/// The cursor styles that can be selected, in menu-cycle order.
pub const CURSOR_STYLES: [&str; 7] = [
    "DefaultUserShape",
    "BlinkingBlock",
    "SteadyBlock",
    "BlinkingUnderScore",
    "SteadyUnderScore",
    "BlinkingBar",
    "SteadyBar",
];

impl CursorKind {
    pub fn from_name(name: Option<&str>) -> Self {
        let style = match name {
            Some("BlinkingBlock") => SetCursorStyle::BlinkingBlock,
            Some("SteadyBlock") => SetCursorStyle::SteadyBlock,
            Some("BlinkingUnderScore") => SetCursorStyle::BlinkingUnderScore,
            Some("SteadyUnderScore") => SetCursorStyle::SteadyUnderScore,
            Some("BlinkingBar") => SetCursorStyle::BlinkingBar,
            Some("SteadyBar") => SetCursorStyle::SteadyBar,
            _ => SetCursorStyle::DefaultUserShape,
        };
        CursorKind { style }
    }
}

impl Default for CursorKind {
    fn default() -> Self {
        CursorKind {
            style: SetCursorStyle::DefaultUserShape,
        }
    }
}
