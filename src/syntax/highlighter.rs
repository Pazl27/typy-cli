use tree_sitter_highlight::{Highlighter, HighlightEvent};

use super::capture;
use super::languages;
use super::Token;

pub fn highlight(source: &str, language: &str) -> Vec<Token> {
    let char_count = source.chars().count();
    let mut tokens = vec![Token::Plain; char_count];

    let Some(config) = languages::config_for(language) else {
        return tokens;
    };

    let bytes = source.as_bytes();
    let byte_to_char = byte_to_char_map(source, char_count);

    let mut highlighter = Highlighter::new();
    let Ok(events) = highlighter.highlight(config, bytes, None, |_| None) else {
        return tokens;
    };

    let mut current = Token::Plain;
    for event in events.flatten() {
        match event {
            HighlightEvent::HighlightStart(highlight) => {
                current = capture::token_for(highlight.0);
            }
            HighlightEvent::HighlightEnd => {
                current = Token::Plain;
            }
            HighlightEvent::Source { start, end } => {
                if current == Token::Plain {
                    continue;
                }
                for token in tokens
                    .iter_mut()
                    .take(byte_to_char[end])
                    .skip(byte_to_char[start])
                {
                    *token = current;
                }
            }
        }
    }

    tokens
}

fn byte_to_char_map(source: &str, char_count: usize) -> Vec<usize> {
    let mut map = vec![char_count; source.len() + 1];
    for (char_index, (byte_index, _)) in source.char_indices().enumerate() {
        map[byte_index] = char_index;
    }
    map[source.len()] = char_count;
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust_keyword_and_string() {
        let src = "fn main() { let x = \"hi\"; }";
        let t = highlight(src, "rust");
        assert_eq!(t.len(), src.chars().count());
        assert_eq!(t[0], Token::Keyword);
        assert_eq!(t[1], Token::Keyword);
        let q = src.chars().position(|c| c == '"').unwrap();
        assert_eq!(t[q], Token::StringLit);
    }

    #[test]
    fn every_registered_language_builds_and_highlights() {
        let cases = [
            ("python", "def f():\n    return 1\n"),
            ("go", "func main() { return }"),
            ("c", "int main() { return 0; }"),
            ("cpp", "int main() { return 0; }"),
            ("javascript", "function f() { return 1; }"),
        ];
        for (language, src) in cases {
            let t = highlight(src, language);
            assert_eq!(t.len(), src.chars().count(), "{language}");
            assert!(
                t.iter().any(|&x| x == Token::Keyword),
                "{language} produced no keywords"
            );
        }
    }

    #[test]
    fn aliases_resolve() {
        assert!(highlight("const x = 1;", "js")
            .iter()
            .any(|&t| t == Token::Keyword));
        assert!(highlight("int f() { return 0; }", "c++")
            .iter()
            .any(|&t| t == Token::Keyword));
    }

    #[test]
    fn unknown_language_is_plain() {
        let src = "fn main() {}";
        let t = highlight(src, "cobol");
        assert!(t.iter().all(|&x| x == Token::Plain));
    }

    #[test]
    fn multibyte_source_stays_aligned() {
        let src = "let \u{00e9} = 1;";
        let t = highlight(src, "rust");
        assert_eq!(t.len(), src.chars().count());
        assert_eq!(t[0], Token::Keyword);
    }
}
