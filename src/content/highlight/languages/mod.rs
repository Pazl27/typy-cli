mod c;
mod cpp;
mod go;
mod javascript;
mod python;
mod rust;

use tree_sitter_highlight::HighlightConfiguration;

const ALIASES: &[(&str, &str)] = &[("c++", "cpp"), ("js", "javascript"), ("ts", "javascript")];

pub fn config_for(language: &str) -> Option<&'static HighlightConfiguration> {
    let key = ALIASES
        .iter()
        .find(|(alias, _)| *alias == language)
        .map(|(_, canonical)| *canonical)
        .unwrap_or(language);

    match key {
        "rust" => Some(&rust::CONFIG),
        "python" => Some(&python::CONFIG),
        "go" => Some(&go::CONFIG),
        "c" => Some(&c::CONFIG),
        "cpp" => Some(&cpp::CONFIG),
        "javascript" => Some(&javascript::CONFIG),
        _ => None,
    }
}
