mod finder;

use anyhow::Result;
use finder::find;
use std::collections::BTreeSet;
use std::path::PathBuf;

const LENGTH: i32 = 70;

pub fn get_words(language: &str) -> Result<Vec<Vec<String>>> {
    let mut words = Vec::new();
    for _ in 0..3 {
        words.push(find(language, LENGTH)?);
    }
    Ok(words)
}

pub fn available_languages() -> Vec<String> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/typy"));
    }
    dirs.push(PathBuf::from("./resources/lang"));

    let mut languages = BTreeSet::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("txt") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    languages.insert(stem.to_string());
                }
            }
        }
    }

    if languages.is_empty() {
        languages.insert("english".to_string());
    }
    languages.into_iter().collect()
}

#[cfg(test)]
mod word_provider_tests {
    use super::*;

    #[test]
    fn test_get_words() {
        let words = get_words("english");

        for word in &words.unwrap() {
            let mut length = 0;
            for w in word {
                length += w.chars().count() as i32;
            }
            assert!(length <= LENGTH);
        }
    }
}
