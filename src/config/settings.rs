//! In-memory, editable snapshot of all user settings.
//!
//! This is the single source of truth while the app is running. It is loaded
//! once from the config file, edited live by the settings screen, and written
//! back to `~/.config/typy/config.toml` on save. Typed getters reuse the
//! existing parsers so the rest of the app keeps working with `ThemeColors`,
//! `KeybindPreset`, etc.

use anyhow::{Context, Result};
use dirs::home_dir;
use std::fs;

use crate::config::graph_colors::Graph;
use crate::config::theme::ThemeColors;
use crate::config::toml_parser::{
    get_config, ConfigToml, CursorTable, GraphTable, KeybindsTable, LanguageTable, ModesTable,
    ThemeTable,
};
use crate::mode::ModeType;
use crate::terminal::KeybindPreset;

use super::cursor_style::CursorKind;
use super::mode_settings::ModeSettings;

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub theme_fg: String,
    pub theme_missing: String,
    pub theme_error: String,
    pub theme_accent: String,

    pub graph_data: String,
    pub graph_title: String,
    pub graph_axis: String,

    pub cursor_style: String,

    pub default_mode: String,
    pub uppercase_chance: f32,
    pub punctuation_chance: f32,
    pub numbers_chance: f32,

    pub language: String,
    pub keybind_preset: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme_fg: "#FFFFFF".into(),
            theme_missing: "#808080".into(),
            theme_error: "#FF0000".into(),
            theme_accent: "#FFFF00".into(),
            graph_data: "#FFFF00".into(),
            graph_title: "#FF0000".into(),
            graph_axis: "#FFFFFF".into(),
            cursor_style: "DefaultUserShape".into(),
            default_mode: "normal".into(),
            uppercase_chance: 0.2,
            punctuation_chance: 0.2,
            numbers_chance: 0.3,
            language: "english".into(),
            keybind_preset: "10ff".into(),
        }
    }
}

impl Settings {
    /// Load the current settings from the parsed config, filling defaults.
    pub fn load() -> Self {
        let cfg = get_config().lock().unwrap();
        let mut s = Settings::default();

        if let Some(t) = cfg.get_theme() {
            if let Some(v) = t.fg {
                s.theme_fg = v;
            }
            if let Some(v) = t.missing {
                s.theme_missing = v;
            }
            if let Some(v) = t.error {
                s.theme_error = v;
            }
            if let Some(v) = t.accent {
                s.theme_accent = v;
            }
        }
        if let Some(g) = cfg.get_graph() {
            if let Some(v) = g.data {
                s.graph_data = v;
            }
            if let Some(v) = g.title {
                s.graph_title = v;
            }
            if let Some(v) = g.axis {
                s.graph_axis = v;
            }
        }
        if let Some(c) = cfg.get_cursor() {
            if let Some(v) = c.style {
                s.cursor_style = v;
            }
        }
        if let Some(m) = cfg.get_modes() {
            if let Some(v) = m.default_mode {
                s.default_mode = v;
            }
            if let Some(v) = m.uppercase_chance.and_then(|c| c.parse().ok()) {
                s.uppercase_chance = v;
            }
            if let Some(v) = m.punctuation_chance.and_then(|c| c.parse().ok()) {
                s.punctuation_chance = v;
            }
            if let Some(v) = m.numbers_chance.and_then(|c| c.parse().ok()) {
                s.numbers_chance = v;
            }
        }
        if let Some(l) = cfg.get_language() {
            if let Some(v) = l.lang {
                s.language = v;
            }
        }
        if let Some(k) = cfg.get_keybinds() {
            if let Some(v) = k.preset {
                s.keybind_preset = v;
            }
        }
        s
    }

    /// Build the serializable config document from these settings.
    pub fn to_config_toml(&self) -> ConfigToml {
        ConfigToml::from_parts(
            ThemeTable {
                fg: Some(self.theme_fg.clone()),
                missing: Some(self.theme_missing.clone()),
                error: Some(self.theme_error.clone()),
                accent: Some(self.theme_accent.clone()),
            },
            GraphTable {
                data: Some(self.graph_data.clone()),
                title: Some(self.graph_title.clone()),
                axis: Some(self.graph_axis.clone()),
            },
            CursorTable {
                style: Some(self.cursor_style.clone()),
            },
            ModesTable {
                default_mode: Some(self.default_mode.clone()),
                uppercase_chance: Some(self.uppercase_chance.to_string()),
                punctuation_chance: Some(self.punctuation_chance.to_string()),
                numbers_chance: Some(self.numbers_chance.to_string()),
            },
            LanguageTable {
                lang: Some(self.language.clone()),
            },
            KeybindsTable {
                preset: Some(self.keybind_preset.clone()),
            },
        )
    }

    /// Persist the settings to `~/.config/typy/config.toml`.
    pub fn save(&self) -> Result<()> {
        let body = self
            .to_config_toml()
            .to_toml_string()
            .context("Failed to serialize config")?;
        let home = home_dir().context("Failed to get home directory")?;
        let dir = home.join(".config/typy");
        fs::create_dir_all(&dir).context("Failed to create config directory")?;
        fs::write(dir.join("config.toml"), body).context("Failed to write config file")?;
        Ok(())
    }

    // --- typed getters used by the running app --------------------------

    pub fn theme(&self) -> ThemeColors {
        ThemeColors::from_opts(
            Some(self.theme_fg.clone()),
            Some(self.theme_missing.clone()),
            Some(self.theme_error.clone()),
            Some(self.theme_accent.clone()),
        )
    }

    pub fn graph(&self) -> Graph {
        Graph::from_opts(
            Some(self.graph_data.clone()),
            Some(self.graph_title.clone()),
            Some(self.graph_axis.clone()),
        )
    }

    pub fn cursor_kind(&self) -> CursorKind {
        CursorKind::from_name(Some(&self.cursor_style))
    }

    pub fn keybind_preset(&self) -> KeybindPreset {
        KeybindPreset::from_str(&self.keybind_preset)
    }

    pub fn language(&self) -> String {
        self.language.clone()
    }

    pub fn mode_settings(&self) -> ModeSettings {
        let default_modes: Vec<ModeType> = crate::mode::parse_modes(&self.default_mode);
        ModeSettings::from_values(
            default_modes,
            self.uppercase_chance,
            self.punctuation_chance,
            self.numbers_chance,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_roundtrip_preserves_values() {
        let mut s = Settings::default();
        s.theme_fg = "#123456".into();
        s.keybind_preset = "monkeytype".into();
        s.numbers_chance = 0.45;

        let body = s.to_config_toml().to_toml_string().unwrap();
        let parsed: ConfigToml = toml::from_str(&body).unwrap();

        assert_eq!(parsed.get_theme().unwrap().fg.unwrap(), "#123456");
        assert_eq!(parsed.get_keybinds().unwrap().preset.unwrap(), "monkeytype");
        assert_eq!(
            parsed.get_modes().unwrap().numbers_chance.unwrap(),
            "0.45"
        );
    }
}
