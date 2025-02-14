use rand::Rng;

#[derive(Debug, PartialEq)]
pub enum ModeType {
    Normal,
    Uppercase,
    Punctuation,
}

#[derive(Debug)]
pub struct Mode {
    modes: Vec<ModeType>,
    pub duration: u64
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

    pub fn transform(&self, list: &mut Vec<Vec<String>>) {
        let mut rng = rand::rng();
        let punctuations = vec![".", ",", "!", "?", ";", ":", "-"];

        for mode in &self.modes {
            match mode {
                ModeType::Uppercase => {
                    for sublist in list.iter_mut() {
                        for item in sublist.iter_mut() {
                            let mut new_item = String::new();
                            for c in item.chars() {
                                if rng.random_bool(0.2) {
                                    new_item.push(c.to_uppercase().next().unwrap());
                                } else {
                                    new_item.push(c);
                                }
                            }
                            *item = new_item;
                        }
                    }
                }
                ModeType::Punctuation => {
                    for sublist in list.iter_mut() {
                        let len = sublist.len();
                        if len > 1 {
                            for i in 0..len - 1 {
                                if rng.random_bool(0.2) {
                                    let punctuation = punctuations[rng.random_range(0..punctuations.len())];
                                    sublist[i].push_str(punctuation);
                                }
                            }
                        }
                    }
                }
                ModeType::Normal => {
                }
            }
        }
    }
}

#[cfg(test)]
mod mode_tests {
    use super::*;

    #[test]
    fn test_from_str_valid_mode() {
        let mode = Mode::from_str(vec!["normal", "uppercase", "punctuation"]).unwrap();
        assert_eq!(mode.modes, vec![ModeType::Normal]);
    }

    #[test]
    fn test_from_str_valid_modes() {
        let mode = Mode::from_str(vec!["uppercase", "punctuation"]).unwrap();
        assert_eq!(mode.modes, vec![ModeType::Uppercase, ModeType::Punctuation]);
    }

    #[test]
    fn test_from_str_invalid_mode() {
        let mode = Mode::from_str(vec!["invalid"]);
        assert!(mode.is_err());
    }

    #[test]
    fn test_from_str_default_to_normal() {
        let mode = Mode::from_str(vec![]).unwrap();
        assert_eq!(mode.modes, vec![ModeType::Normal]);
    }

    #[test]
    fn test_add_duration() {
        let mode = Mode::from_str(vec!["normal"]).unwrap().add_duration(10);
        assert_eq!(mode.modes, vec![ModeType::Normal]);
        assert_eq!(mode.duration, 10);
    }

    #[test]
    fn test_transform_uppercase() {
        let mode = Mode::from_str(vec!["uppercase"]).unwrap();
        let mut list = vec![vec!["hello".to_string(), "world".to_string()]];
        mode.transform(&mut list);
        // Since the transformation is random, we can't assert exact values, but we can check the structure
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].len(), 2);
    }

    #[test]
    fn test_transform_punctuation() {
        let mode = Mode::from_str(vec!["punctuation"]).unwrap();
        let mut list = vec![vec!["hello".to_string(), "world".to_string()]];
        mode.transform(&mut list);
        // Since the transformation is random, we can't assert exact values, but we can check the structure
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].len(), 2);
    }
}
