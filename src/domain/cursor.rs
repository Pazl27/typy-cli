use crossterm::cursor::SetCursorStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    BlinkingBlock,
    Underline,
    BlinkingUnderline,
    Bar,
    BlinkingBar,
}

impl CursorStyle {
    pub const ALL: [CursorStyle; 6] = [
        CursorStyle::Block,
        CursorStyle::BlinkingBlock,
        CursorStyle::Underline,
        CursorStyle::BlinkingUnderline,
        CursorStyle::Bar,
        CursorStyle::BlinkingBar,
    ];

    pub fn token(self) -> &'static str {
        match self {
            CursorStyle::Block => "block",
            CursorStyle::BlinkingBlock => "blinking block",
            CursorStyle::Underline => "underline",
            CursorStyle::BlinkingUnderline => "blinking underline",
            CursorStyle::Bar => "bar",
            CursorStyle::BlinkingBar => "blinking bar",
        }
    }

    pub fn from_token(token: &str) -> Option<CursorStyle> {
        CursorStyle::ALL
            .into_iter()
            .find(|style| style.token() == token)
    }

    pub fn to_set_cursor_style(self) -> SetCursorStyle {
        match self {
            CursorStyle::Block => SetCursorStyle::SteadyBlock,
            CursorStyle::BlinkingBlock => SetCursorStyle::BlinkingBlock,
            CursorStyle::Underline => SetCursorStyle::SteadyUnderScore,
            CursorStyle::BlinkingUnderline => SetCursorStyle::BlinkingUnderScore,
            CursorStyle::Bar => SetCursorStyle::SteadyBar,
            CursorStyle::BlinkingBar => SetCursorStyle::BlinkingBar,
        }
    }
}
