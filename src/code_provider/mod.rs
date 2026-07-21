use anyhow::{bail, Context, Result};
use dirs::home_dir;
use rand::rng;
use rand::seq::IndexedRandom;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;

const DATASET_URL: &str =
    "https://raw.githubusercontent.com/Pazl27/typy-cli/refs/heads/master/resources/code/snippets.json";

#[derive(Debug, Clone, Deserialize)]
pub struct Snippet {
    pub language: String,
    pub path: String,
    pub url: String,
    pub content: String,
}

pub fn available_languages() -> Vec<String> {
    ["c", "cpp", "go", "javascript", "python", "rust"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

pub fn get_snippet(selector: &str) -> Result<Snippet> {
    let dataset = load_dataset()?;
    if dataset.is_empty() {
        bail!("No code snippets available");
    }

    let pool: Vec<&Snippet> = if selector.is_empty() || selector == "any" {
        dataset.iter().collect()
    } else {
        let filtered: Vec<&Snippet> = dataset.iter().filter(|s| s.language == selector).collect();
        if filtered.is_empty() {
            dataset.iter().collect()
        } else {
            filtered
        }
    };

    let mut rng = rng();
    let snippet = pool.choose(&mut rng).context("No code snippets available")?;
    Ok((*snippet).clone())
}

fn cache_path() -> Result<PathBuf> {
    let home = home_dir().context("Failed to get home directory")?;
    Ok(home.join(".local/share/typy/code/snippets.json"))
}

fn load_dataset() -> Result<Vec<Snippet>> {
    if let Ok(text) = read_to_string("./resources/code/snippets.json") {
        if let Ok(dataset) = serde_json::from_str(&text) {
            return Ok(dataset);
        }
    }

    let cache = cache_path()?;
    if let Ok(text) = read_to_string(&cache) {
        if let Ok(dataset) = serde_json::from_str(&text) {
            return Ok(dataset);
        }
    }

    let text = download_dataset()?;
    if let Some(parent) = cache.parent() {
        create_dir_all(parent).ok();
    }
    write(&cache, &text).ok();
    serde_json::from_str(&text).context("Failed to parse downloaded snippets")
}

fn download_dataset() -> Result<String> {
    let client = Client::builder()
        .user_agent("typy-cli")
        .build()
        .context("Failed to build HTTP client")?;
    let resp = client
        .get(DATASET_URL)
        .send()
        .context("Failed to download code snippets (check your internet connection)")?;
    if !resp.status().is_success() {
        bail!("Failed to download code snippets ({})", resp.status());
    }
    resp.text().context("Failed to read snippet download")
}
