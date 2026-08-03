use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModeType {
    Normal,
    Uppercase,
    Punctuation,
    Numbers,
}

impl ModeType {
    pub fn token(self) -> &'static str {
        match self {
            ModeType::Normal => "normal",
            ModeType::Uppercase => "uppercase",
            ModeType::Punctuation => "punctuation",
            ModeType::Numbers => "numbers",
        }
    }
}

impl FromStr for ModeType {
    type Err = ();

    fn from_str(input: &str) -> Result<ModeType, Self::Err> {
        match input {
            "normal" => Ok(ModeType::Normal),
            "uppercase" => Ok(ModeType::Uppercase),
            "punctuation" => Ok(ModeType::Punctuation),
            "numbers" => Ok(ModeType::Numbers),
            _ => Err(()),
        }
    }
}
