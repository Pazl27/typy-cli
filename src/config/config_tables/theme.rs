use crossterm::style::Color;

use crate::config::toml_parser::get_config;

#[derive(Debug)]
pub struct ThemeColors {
    pub fg: Color,
    pub missing: Color,
    pub error: Color,
    pub accent: Color,
}

impl ThemeColors {
    pub fn new() -> Self {
        let theme_colors: ThemeColors = match get_config().lock().unwrap().get_theme() {
            Some(colors) => {
                let fg = colors
                    .fg
                    .and_then(|c| hex_to_rgb(&c))
                    .unwrap_or(Color::White);
                let missing = colors
                    .missing
                    .and_then(|c| hex_to_rgb(&c))
                    .unwrap_or(Color::Grey);
                let error = colors
                    .error
                    .and_then(|c| hex_to_rgb(&c))
                    .unwrap_or(Color::Red);
                let accent = colors
                    .accent
                    .and_then(|c| hex_to_rgb(&c))
                    .unwrap_or(Color::Yellow);

                ThemeColors {
                    fg,
                    missing,
                    error,
                    accent,
                }
            }
            None => ThemeColors::default(),
        };
        theme_colors
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        ThemeColors {
            fg: Color::White,
            missing: Color::Grey,
            error: Color::Red,
            accent: Color::Yellow,
        }
    }
}

fn hex_to_rgb(hex: &str) -> Option<Color> {
    if hex.len() == 7 && hex.starts_with('#') {
        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
        Some(Color::Rgb { r, g, b })
    } else {
        None
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(
            hex_to_rgb("#ffffff"),
            Some(Color::Rgb {
                r: 255,
                g: 255,
                b: 255
            })
        );
        assert_eq!(hex_to_rgb("#000000"), Some(Color::Rgb { r: 0, g: 0, b: 0 }));
        assert_eq!(
            hex_to_rgb("#ff0000"),
            Some(Color::Rgb { r: 255, g: 0, b: 0 })
        );
        assert_eq!(
            hex_to_rgb("#00ff00"),
            Some(Color::Rgb { r: 0, g: 255, b: 0 })
        );
        assert_eq!(
            hex_to_rgb("#0000ff"),
            Some(Color::Rgb { r: 0, g: 0, b: 255 })
        );
        assert_eq!(
            hex_to_rgb("#123456"),
            Some(Color::Rgb {
                r: 18,
                g: 52,
                b: 86
            })
        );
        assert_eq!(
            hex_to_rgb("#abcdef"),
            Some(Color::Rgb {
                r: 171,
                g: 205,
                b: 239
            })
        );
        assert_eq!(hex_to_rgb("#12345"), None);
        assert_eq!(hex_to_rgb("#1234567"), None);
        assert_eq!(hex_to_rgb("123456"), None);
        assert_eq!(hex_to_rgb("#12345g"), None);
    }
}
