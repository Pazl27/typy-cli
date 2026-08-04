use std::sync::LazyLock;

use tree_sitter_highlight::HighlightConfiguration;

use super::super::capture::HIGHLIGHT_NAMES;

pub static CONFIG: LazyLock<HighlightConfiguration> = LazyLock::new(|| {
    let query = format!(
        "{}\n{}",
        tree_sitter_c::HIGHLIGHT_QUERY,
        tree_sitter_cpp::HIGHLIGHT_QUERY
    );
    let mut config =
        HighlightConfiguration::new(tree_sitter_cpp::LANGUAGE.into(), "cpp", &query, "", "")
            .expect("valid cpp highlight configuration");
    config.configure(HIGHLIGHT_NAMES);
    config
});
