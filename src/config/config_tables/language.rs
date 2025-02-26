use crate::config::toml_parser::get_config;

pub struct Language {
    pub lang: String,
}

impl Language {
    pub fn new() -> Self {
        let theme_colors: Language = match get_config().lock().unwrap().get_language() {
            Some(language) => {
                let lang = language.lang.unwrap_or("english".to_string());

                Language { lang }
            }
            None => Language::default(),
        };
        theme_colors
    }
}

impl Default for Language {
    fn default() -> Self {
        Language {
            lang: "english".to_string(),
        }
    }
}
