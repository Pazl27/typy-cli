use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum GameMode {
    Text,
    Code,
}

impl GameMode {
    pub fn content_token(self) -> &'static str {
        match self {
            GameMode::Text => "words",
            GameMode::Code => "code",
        }
    }

    pub fn is_code_content(content: &str) -> bool {
        content == GameMode::Code.content_token()
    }
}
