use ratatui::style::Color;

pub struct Graph {
    pub data: Color,
    pub title: Color,
    pub axis: Color,
}

impl Graph {
    pub fn from_opts(
        data: Option<String>,
        title: Option<String>,
        axis: Option<String>,
    ) -> Self {
        Graph {
            data: data.and_then(|c| hex_to_rgb(&c)).unwrap_or(Color::Yellow),
            title: title.and_then(|c| hex_to_rgb(&c)).unwrap_or(Color::Red),
            axis: axis.and_then(|c| hex_to_rgb(&c)).unwrap_or(Color::White),
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Graph {
            data: Color::Yellow,
            title: Color::Red,
            axis: Color::White,
        }
    }
}

fn hex_to_rgb(hex: &str) -> Option<Color> {
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
mod graph_tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb("#FFFFFF"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(hex_to_rgb("#000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(hex_to_rgb("#FF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(hex_to_rgb("#00FF00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(hex_to_rgb("#0000FF"), Some(Color::Rgb(0, 0, 255)));
        assert_eq!(hex_to_rgb("#123456"), Some(Color::Rgb(18, 52, 86)));
        assert_eq!(hex_to_rgb("123456"), None);
        assert_eq!(hex_to_rgb("#12345G"), None);
        assert_eq!(hex_to_rgb("#12345"), None);
    }
}
