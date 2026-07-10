//! Built-in quotes for `quotes` mode.
//!
//! A quote is a fixed sentence rather than random words, so it is generated
//! outside the normal word-pool path. Kept small and inline for now; can later
//! be sourced from a downloadable `quotes_<lang>.txt` like the word lists.

use rand::seq::IndexedRandom;

const ENGLISH: &[&str] = &[
    "The quick brown fox jumps over the lazy dog.",
    "Simplicity is the ultimate sophistication.",
    "The only way to do great work is to love what you do.",
    "Whether you think you can or you think you can't, you're right.",
    "It always seems impossible until it is done.",
    "The best time to plant a tree was twenty years ago; the second best time is now.",
];

const FRENCH: &[&str] = &[
    "Le vent se lève, il faut tenter de vivre.",
    "On ne voit bien qu'avec le coeur, l'essentiel est invisible pour les yeux.",
    "La simplicité est la sophistication suprême.",
    "Rien ne sert de courir, il faut partir à point.",
    "Le doute est le commencement de la sagesse.",
    "Chaque jour est une nouvelle chance de changer sa vie.",
];

/// Return a random quote for the given language (falls back to English).
pub fn random_quote(language: &str) -> String {
    let pool = match language.to_lowercase().as_str() {
        "french" | "français" | "francais" => FRENCH,
        _ => ENGLISH,
    };
    let mut rng = rand::rng();
    pool.choose(&mut rng)
        .copied()
        .unwrap_or("The quick brown fox jumps over the lazy dog.")
        .to_string()
}
