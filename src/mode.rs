#[derive(Debug, PartialEq)]
pub enum ModeType {
    Normal,
    Uppercase,
    Punctuation,
}

#[derive(Debug)]
pub struct Mode {
    modes: Vec<ModeType>,
    duration: u64
}

impl Mode {
    pub fn from_str(mode_strs: Vec<&str>) -> Result<Self, String> {
        let mut modes = Vec::new();

        for mode_str in mode_strs {
            match mode_str {
                "normal" => modes.push(ModeType::Normal),
                "uppercase" => modes.push(ModeType::Uppercase),
                "punctuation" => modes.push(ModeType::Punctuation),
                _ => return Err(format!("Invalid mode: {}", mode_str)),
            }
        }

        // If no specific mode is provided, default to normal
        modes.is_empty().then(|| {
            modes.push(ModeType::Normal);
        });

        modes.contains(&ModeType::Normal).then(|| {
            modes.clear();
            modes.push(ModeType::Normal);
        });

        Ok(Mode { modes, duration: 0 })
    }
    pub fn add_duration(mut self, duration: u64) -> Self {
        self.duration = duration;
        self
    }
}
