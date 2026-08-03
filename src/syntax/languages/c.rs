use std::sync::LazyLock;

use tree_sitter_highlight::HighlightConfiguration;

use super::super::capture::HIGHLIGHT_NAMES;

pub static CONFIG: LazyLock<HighlightConfiguration> = LazyLock::new(|| {
    let mut config = HighlightConfiguration::new(
        tree_sitter_c::LANGUAGE.into(),
        "c",
        tree_sitter_c::HIGHLIGHT_QUERY,
        "",
        "",
    )
    .expect("valid c highlight configuration");
    config.configure(HIGHLIGHT_NAMES);
    config
});
