use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;
use std::fs::{self, File};

pub mod display;

#[derive(Debug, Serialize, Deserialize)]
pub struct Score {
    timestamp: NaiveDateTime,
    wpm: u32,
    raw: u32,
    accuracy: f32,
}

impl Score {
    pub fn new(wpm: u32, raw: u32, accuracy: f32) -> Score {
        Score {
            timestamp: chrono::Local::now().naive_local(),
            wpm,
            raw,
            accuracy,
        }
    }

    pub fn sort_scores(scores: &mut Vec<Score>) {
            scores.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    pub fn save_score(score: Score) -> Result<()> {
        let mut scores = Self::get_scores()?;
        scores.push(score);

        if scores.len() > 10 {
            Self::sort_scores(&mut scores);
            Self::cleanup_scores(&mut scores);
        }

        Self::write_to_file(scores)?;

        Ok(())
    }

    fn cleanup_scores(scores: &mut Vec<Score>) {
        scores.truncate(10);
    }

    pub fn get_scores() -> Result<Vec<Score>> {
        let mut path = dirs::home_dir().context("Failed to get home directory")?;
        path.push(".local/share/typy/scores.json");

        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).context("Failed to create directories")?;
            }
            File::create(&path).context("Failed to create scores.json file")?;
        }

        let file = File::open(&path).context("Failed to open scores.json file")?;
        let scores: Vec<Score> = match serde_json::from_reader(file) {
            Ok(scores) => scores,
            Err(e) if e.is_eof() => Vec::new(),
            Err(e) => return Err(e).context("Failed to read scores from file"),
        };
        Ok(scores)
    }

    fn write_to_file(scores: Vec<Score>) -> Result<()> {
        let mut path = dirs::home_dir().context("Failed to get home directory")?;
        path.push(".local/share/typy/scores.json");

        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist"));
        }

        let mut file = File::create(&path).context("Failed to truncate scores.json file")?;
        to_writer_pretty(&mut file, &scores).context("Failed to write scores to file")?;

        Ok(())
    }
}
