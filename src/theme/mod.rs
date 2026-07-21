use std::collections::{BTreeMap, BTreeSet};

use dirs::home_dir;
use include_dir::{include_dir, Dir};
use ratatui::style::Color;
use serde::Deserialize;

static BUILTIN: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources/themes");

pub const DEFAULT_THEME: &str = "Catppuccin Mocha";

#[derive(Clone)]
pub struct Theme {
    pub name: String,
    pub fg: Color,
    pub missing: Color,
    pub error: Color,
    pub accent: Color,
    pub graph_data: Color,
    pub graph_title: Color,
    pub graph_axis: Color,
}

#[derive(Deserialize)]
struct ThemeSpec {
    name: Option<String>,
    fg: Option<String>,
    missing: Option<String>,
    error: Option<String>,
    accent: Option<String>,
    graph_data: Option<String>,
    graph_title: Option<String>,
    graph_axis: Option<String>,
}

impl ThemeSpec {
    fn into_theme(self, fallback_name: &str) -> Theme {
        let fg = parse_color(self.fg).unwrap_or(Color::White);
        let missing = parse_color(self.missing).unwrap_or(Color::Gray);
        let error = parse_color(self.error).unwrap_or(Color::Red);
        let accent = parse_color(self.accent).unwrap_or(Color::Yellow);
        Theme {
            name: self.name.unwrap_or_else(|| fallback_name.to_string()),
            fg,
            missing,
            error,
            accent,
            graph_data: parse_color(self.graph_data).unwrap_or(accent),
            graph_title: parse_color(self.graph_title).unwrap_or(error),
            graph_axis: parse_color(self.graph_axis).unwrap_or(missing),
        }
    }
}

pub fn load(name: &str) -> Theme {
    if let Some(theme) = user_themes().into_iter().find(|t| t.name == name) {
        return theme;
    }
    if let Some(theme) = builtin_themes().into_iter().find(|t| t.name == name) {
        return theme;
    }
    builtin_themes()
        .into_iter()
        .find(|t| t.name == DEFAULT_THEME)
        .unwrap_or_else(fallback_theme)
}

pub fn available_themes() -> Vec<String> {
    let mut names = BTreeSet::new();
    for theme in builtin_themes() {
        names.insert(theme.name);
    }
    for theme in user_themes() {
        names.insert(theme.name);
    }
    names.into_iter().collect()
}

fn builtin_themes() -> Vec<Theme> {
    BUILTIN
        .files()
        .filter_map(|file| {
            let text = file.contents_utf8()?;
            let spec: ThemeSpec = toml::from_str(text).ok()?;
            let stem = file.path().file_stem()?.to_str()?;
            Some(spec.into_theme(stem))
        })
        .collect()
}

fn user_themes() -> Vec<Theme> {
    let Some(path) = home_dir().map(|p| p.join(".config/typy/theme.toml")) else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(map) = toml::from_str::<BTreeMap<String, ThemeSpec>>(&text) else {
        return Vec::new();
    };
    map.into_iter()
        .map(|(key, spec)| spec.into_theme(&key))
        .collect()
}

fn fallback_theme() -> Theme {
    Theme {
        name: DEFAULT_THEME.to_string(),
        fg: Color::Rgb(205, 214, 244),
        missing: Color::Rgb(108, 112, 134),
        error: Color::Rgb(243, 139, 168),
        accent: Color::Rgb(249, 226, 175),
        graph_data: Color::Rgb(166, 227, 161),
        graph_title: Color::Rgb(203, 166, 247),
        graph_axis: Color::Rgb(88, 91, 112),
    }
}

fn parse_color(hex: Option<String>) -> Option<Color> {
    let hex = hex?;
    if hex.len() == 7 && hex.starts_with('#') {
        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        assert_eq!(
            parse_color(Some("#ffffff".to_string())),
            Some(Color::Rgb(255, 255, 255))
        );
        assert_eq!(
            parse_color(Some("#123456".to_string())),
            Some(Color::Rgb(18, 52, 86))
        );
        assert_eq!(parse_color(Some("#12345".to_string())), None);
        assert_eq!(parse_color(Some("123456".to_string())), None);
        assert_eq!(parse_color(None), None);
    }

    #[test]
    fn test_builtins_load() {
        let themes = builtin_themes();
        assert!(themes.iter().any(|t| t.name == DEFAULT_THEME));
    }

    #[test]
    fn test_default_available() {
        assert!(available_themes().contains(&DEFAULT_THEME.to_string()));
    }
}
