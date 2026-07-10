mod finder;
mod quotes;

use anyhow::Result;
use finder::random_words;

pub use finder::list_languages;
pub use quotes::random_quote;

/// Number of words generated for a time / zen test before top-ups.
pub const DEFAULT_POOL_WORDS: usize = 60;

/// Return `count` random words as a flat list.
pub fn word_pool(language: &str, count: usize) -> Result<Vec<String>> {
    random_words(language, count)
}

#[cfg(test)]
mod word_provider_tests {
    use super::*;

    #[test]
    fn test_word_pool_count() {
        let words = word_pool("english", 25).unwrap();
        assert_eq!(words.len(), 25);
    }
}
