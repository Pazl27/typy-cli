use super::Token;

pub const HIGHLIGHT_NAMES: &[&str] = &[
    "keyword",
    "string",
    "comment",
    "number",
    "function",
    "type",
    "constant",
    "operator",
    "variable",
];

pub fn token_for(index: usize) -> Token {
    match HIGHLIGHT_NAMES.get(index).copied() {
        Some("keyword") => Token::Keyword,
        Some("string") => Token::StringLit,
        Some("comment") => Token::Comment,
        Some("number") | Some("constant") => Token::Number,
        _ => Token::Plain,
    }
}
