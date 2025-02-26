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

pub fn find(language: &str, lenght: i32) -> Result<Vec<String>> {
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

    let words = read_file(words_file.to_str().unwrap())?;
    let mut word = random_word(&words);

    let mut fitted_words = Vec::new();
    while check_if_fits(&word, &mut fitted_words, lenght) {
        fitted_words.push(word.clone());
        word = random_word(&words);
    }

    Ok(fitted_words)
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

fn check_if_fits(word: &str, fitted_words: &mut [String], lenght: i32) -> bool {
    let list_length: i32 = fitted_words
        .iter()
        .map(|s| s.chars().count() as i32)
        .sum::<i32>()
        + word.chars().count() as i32;

    if list_length > lenght {
        return false;
    }
    true
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

    #[test]
    fn test_check_if_fits() {
        let word = "Hello".to_string();
        let mut fitted_words = Vec::new();
        let lenght = 5;
        assert!(check_if_fits(&word, &mut fitted_words, lenght));
        fitted_words.push("Hello".to_string());
        assert!(!check_if_fits(&word, &mut fitted_words, lenght));
    }
}
