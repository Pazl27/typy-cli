use anyhow::Result;
use dirs::home_dir;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use rand::seq::IndexedRandom;

pub fn find(lenght: i32, res: &str) -> Result<Vec<String>, std::io::Error> {
    let path = if res.contains("resources") {
        PathBuf::from(res)
    } else {
        let mut home_path = home_dir().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Home directory not found",
        ))?;
        home_path.push(res);
        home_path
    };

    let words = read_file(path.to_str().unwrap())?;
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

fn random_word(words: &Vec<String>) -> String {
    let mut rng = rand::rng();
    let word = words.choose(&mut rng).unwrap();
    word.to_string()
}

fn check_if_fits(word: &String, fitted_words: &mut Vec<String>, lenght: i32) -> bool {
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
        assert_eq!(check_if_fits(&word, &mut fitted_words, lenght), true);
        fitted_words.push("Hello".to_string());
        assert_eq!(check_if_fits(&word, &mut fitted_words, lenght), false);
    }
}
