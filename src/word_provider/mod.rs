mod finder;

use finder::find;

const LENGTH: i32 = 70;

pub fn get_words() -> Vec<Vec<String>> {
    let mut words = Vec::new();
    for _ in 0..3 {
        words.push(find(LENGTH));
    }
    words
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_get_words() {
        let words = get_words();

        // print them
        for word in &words {
            for w in word {
                print!("{} ", w);
            }
            println!();
        }

        for word in &words {
            let mut length = 0;
            for w in word {
                length += w.chars().count() as i32;
            }
            assert!(length <= LENGTH);
        }
    }
}
