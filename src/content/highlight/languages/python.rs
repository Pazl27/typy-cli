use std::sync::LazyLock;

use tree_sitter_highlight::HighlightConfiguration;

use super::super::capture::HIGHLIGHT_NAMES;

pub static CONFIG: LazyLock<HighlightConfiguration> = LazyLock::new(|| {
    let mut config = HighlightConfiguration::new(
        tree_sitter_python::LANGUAGE.into(),
        "python",
        tree_sitter_python::HIGHLIGHTS_QUERY,
        "",
        "",
    )
    .expect("valid python highlight configuration");
    config.configure(HIGHLIGHT_NAMES);
    config
});
