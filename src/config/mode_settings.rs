use crate::config::toml_parser::get_config;
use crate::mode::ModeType;
use std::str::FromStr;

#[derive(Debug)]
pub struct ModeSettings {
    pub default_modes: Vec<ModeType>,
    pub uppercase_chance: f32,
    pub punctuation_chance: f32,
}

impl ModeSettings {
    pub fn new() -> Self {
        let theme_colors: ModeSettings = match get_config().lock().unwrap().get_modes() {
            Some(settings) => {
                let default_modes = settings
                    .default_mode
                    .map(|m| {
                        let modes: Vec<ModeType> = m.split(',')
                            .filter_map(|mode| ModeType::from_str(mode.trim()).ok())
                            .collect();
                        if modes.contains(&ModeType::Normal) {
                            vec![ModeType::Normal]
                        } else {
                            modes
                        }
                    })
                    .unwrap_or(vec![ModeType::Normal]);
                let uppercase_chance = settings
                    .uppercase_chance
                    .and_then(|c| c.parse::<f32>().ok())
                    .unwrap_or(0.2);
                let punctuation_chance = settings
                    .punctuation_chance
                    .and_then(|c| c.parse::<f32>().ok())
                    .unwrap_or(0.2);

                ModeSettings { default_modes, uppercase_chance, punctuation_chance }
            }
            None => ModeSettings::default(),
        };
        theme_colors
    }
}

impl Default for ModeSettings {
    fn default() -> Self {
        ModeSettings {
            default_modes: vec![ModeType::Normal],
            uppercase_chance: 0.2,
            punctuation_chance: 0.2,
        }
    }
}
