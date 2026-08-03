use std::sync::LazyLock;

use tree_sitter_highlight::HighlightConfiguration;

use super::super::capture::HIGHLIGHT_NAMES;

pub static CONFIG: LazyLock<HighlightConfiguration> = LazyLock::new(|| {
    let mut config = HighlightConfiguration::new(
        tree_sitter_javascript::LANGUAGE.into(),
        "javascript",
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        "",
        "",
    )
    .expect("valid javascript highlight configuration");
    config.configure(HIGHLIGHT_NAMES);
    config
});
