use anyhow::{bail, Context, Result};
use dirs::home_dir;
use reqwest::blocking::get;
use std::{
    fs::{create_dir_all, write, File},
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::LazyLock,
};

use rand::seq::IndexedRandom;

static WORDS_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    if cfg!(test) {
        Some(PathBuf::from("./resources/"))
    } else {
        home_dir().map(|p| p.join(".local/share/typy"))
    }
});
const WORDS_URL: &str =
    "https://raw.githubusercontent.com/Pazl27/typy-cli/refs/heads/master/resources/";

/// Load the full word list for `language`, downloading the language file on
/// first use if it is not already present locally.
pub fn load_words(language: &str) -> Result<Vec<String>> {
    let Some(words_file) = WORDS_DIR
        .as_ref()
        .map(|p| p.join(format!("{language}.txt")))
    else {
        bail!("Unable to find home directory");
    };

    // Download words file if not already present
    if !words_file.exists() {
        create_dir_all(words_file.parent().unwrap())?;
        let language_url = format!("{WORDS_URL}{language}.txt");
        let resp = get(&language_url)
            .context("Failed to download words file from ".to_owned() + &language_url)?;
        write(
            &words_file,
            resp.text()
                .context("Failed to extract text from words file download")?,
        )
        .with_context(|| format!("Failed to save words file to {words_file:#?}"))?;
    }

    read_file(words_file.to_str().unwrap()).map_err(Into::into)
}

/// Pick `count` random words from `language`.
pub fn random_words(language: &str, count: usize) -> Result<Vec<String>> {
    let words = load_words(language)?;
    Ok((0..count).map(|_| random_word(&words)).collect())
}

/// List the available wordlists (the `*.txt` files) in the words directory,
/// by language name (file stem), sorted.
pub fn list_languages() -> Vec<String> {
    let Some(dir) = WORDS_DIR.as_ref() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut langs: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            (path.extension().and_then(|s| s.to_str()) == Some("txt"))
                .then(|| path.file_stem().and_then(|s| s.to_str()).map(String::from))
                .flatten()
        })
        .collect();
    langs.sort();
    langs.dedup();
    langs
}

fn read_file(path: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut words = Vec::new();
    for line in reader.lines() {
        words.push(line?);
    }
    Ok(words)
}

fn random_word(words: &[String]) -> String {
    let mut rng = rand::rng();
    let word = words.choose(&mut rng).unwrap();
    word.to_string()
}

#[cfg(test)]
mod finder_tests {

    use super::*;

    #[test]
    fn test_read_file() {
        let words = read_file("./resources/english.txt").unwrap();
        assert_eq!(words.len(), 7776);
    }

    #[test]
    fn test_random_word() {
        let words = vec!["Hello".to_string(), "World".to_string()];
        let word = random_word(&words);
        assert!(word == "Hello" || word == "World");
    }
}
