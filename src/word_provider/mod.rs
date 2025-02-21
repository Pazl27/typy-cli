mod finder;

use finder::find;
use anyhow::Result;

const LENGTH: i32 = 70;

pub fn get_words(res: &str) -> Result<Vec<Vec<String>>> {
    let mut words = Vec::new();
    for _ in 0..3 {
        words.push(find(LENGTH, res)?);
    }
    Ok(words)
}

#[cfg(test)]

mod word_provider_tests {
    use super::*;

    #[test]
    fn test_get_words() {
        let words = get_words("resources/words.txt");

        for word in &words.unwrap() {
            let mut length = 0;
            for w in word {
                length += w.chars().count() as i32;
            }
            assert!(length <= LENGTH);
        }
    }
}
