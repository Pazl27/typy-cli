use ratatui::style::Color;

use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
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

pub fn tokens_for_line(line: &[char], language: &str) -> Vec<Token> {
    let mut tokens = vec![Token::Plain; line.len()];
    let comment = comment_marker(language);
    let keywords = keywords(language);

    let mut i = 0;
    while i < line.len() {
        let c = line[i];

        if matches(line, i, comment) {
            for t in tokens.iter_mut().skip(i) {
                *t = Token::Comment;
            }
            break;
        }

        if c == '"' || c == '\'' {
            let quote = c;
            tokens[i] = Token::StringLit;
            let mut j = i + 1;
            while j < line.len() {
                tokens[j] = Token::StringLit;
                if line[j] == '\\' && j + 1 < line.len() {
                    tokens[j + 1] = Token::StringLit;
                    j += 2;
                    continue;
                }
                if line[j] == quote {
                    j += 1;
                    break;
                }
                j += 1;
            }
            i = j;
            continue;
        }

        if c.is_ascii_digit() {
            let mut j = i;
            while j < line.len() && (line[j].is_ascii_alphanumeric() || line[j] == '.' || line[j] == '_') {
                tokens[j] = Token::Number;
                j += 1;
            }
            i = j;
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let start = i;
            let mut j = i;
            while j < line.len() && (line[j].is_alphanumeric() || line[j] == '_') {
                j += 1;
            }
            let word: String = line[start..j].iter().collect();
            if keywords.contains(&word.as_str()) {
                for t in tokens.iter_mut().take(j).skip(start) {
                    *t = Token::Keyword;
                }
            }
            i = j;
            continue;
        }

        i += 1;
    }

    tokens
}

fn matches(line: &[char], at: usize, marker: &str) -> bool {
    let marker: Vec<char> = marker.chars().collect();
    if marker.is_empty() || at + marker.len() > line.len() {
        return false;
    }
    line[at..at + marker.len()] == marker[..]
}

fn comment_marker(language: &str) -> &'static str {
    match language {
        "python" | "perl" | "ruby" | "bash" | "sh" => "#",
        _ => "//",
    }
}

fn keywords(language: &str) -> &'static [&'static str] {
    match language {
        "python" => &[
            "def", "return", "if", "elif", "else", "for", "while", "in", "and", "or", "not", "import",
            "from", "as", "class", "try", "except", "finally", "with", "lambda", "yield", "pass",
            "break", "continue", "None", "True", "False", "range", "len", "print",
        ],
        "rust" => &[
            "fn", "let", "mut", "if", "else", "for", "while", "loop", "match", "return", "struct",
            "enum", "impl", "trait", "pub", "use", "mod", "const", "static", "ref", "move", "where",
            "as", "in", "self", "Self", "Some", "None", "Ok", "Err", "true", "false", "u32", "u64",
            "i32", "i64", "usize", "bool", "str",
        ],
        "go" => &[
            "func", "var", "const", "if", "else", "for", "range", "return", "package", "import",
            "type", "struct", "interface", "map", "chan", "go", "defer", "switch", "case", "default",
            "break", "continue", "int", "string", "bool", "nil", "true", "false",
        ],
        "c" | "cpp" | "c++" => &[
            "int", "char", "float", "double", "void", "long", "short", "unsigned", "signed", "if",
            "else", "for", "while", "do", "return", "struct", "enum", "union", "typedef", "const",
            "static", "sizeof", "break", "continue", "switch", "case", "default", "bool", "true",
            "false",
        ],
        "javascript" | "js" | "typescript" | "ts" => &[
            "function", "let", "const", "var", "if", "else", "for", "while", "do", "return",
            "class", "new", "this", "import", "export", "from", "async", "await", "try", "catch",
            "finally", "switch", "case", "default", "break", "continue", "typeof", "instanceof",
            "null", "undefined", "true", "false",
        ],
        _ => &[],
    }
}
