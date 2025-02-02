mod finder;

use finder::find;

pub fn get_words() -> Vec<String> {
    const LENGTH: i32 = 5;
    find(LENGTH)

}
