mod capture;
mod highlighter;
mod languages;

use ratatui::style::Color;

use crate::view::theme::Theme;

pub use highlighter::highlight;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    Keyword,
    StringLit,
    Comment,
    Number,
    Plain,
}

pub fn token_color(token: Token, theme: &Theme) -> Color {
    match token {
        Token::Keyword => theme.keyword,
        Token::StringLit => theme.string,
        Token::Comment => theme.comment,
        Token::Number => theme.number,
        Token::Plain => theme.fg,
    }
}
