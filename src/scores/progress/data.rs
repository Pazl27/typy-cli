use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;
use std::fs::{self, File};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Averages {
    pub wpm_avg: WpmAvg,
    pub raw_avg: RawAvg,
    pub accuracy_avg: AccuracyAvg,
    #[serde(default)]
    pub consistency_avg: ConsistencyAvg,
}

impl Averages {
    /// Total number of tests ever played (all-time, not just the retained 10).
    pub fn total_tests(&self) -> u32 {
        self.wpm_avg.count
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConsistencyAvg {
    pub avg: f32,
    count: u32,
    sum_all: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WpmAvg {
    pub avg: f32,
    count: u32,
    sum_all: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RawAvg {
    pub avg: f32,
    count: u32,
    sum_all: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccuracyAvg {
    pub avg: f32,
    count: u32,
    sum_all: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub timestamp: NaiveDateTime,
    pub wpm: u32,
    pub raw: u32,
    pub accuracy: f32,
    #[serde(default)]
    pub consistency: f32,
    /// Per-second WPM curve, so the test's graph can be reviewed later.
    #[serde(default)]
    pub series: Vec<i32>,
    /// Mode label the test was played with (e.g. "uppercase punctuation").
    #[serde(default)]
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub scores: Vec<Score>,
    pub averages: Averages,
}

impl Data {
    fn new(scores: Vec<Score>, averages: Averages) -> Self {
        Data { scores, averages }
    }

    pub fn save_data(score: Score) -> Result<()> {
        let scores = Score::update_scores(&score)?;
        let averages = Averages::new(score)?;

        let data = Data::new(scores, averages);
        Self::write_to_file(data)?;
        Ok(())
    }

    pub fn get_data() -> Result<Data> {
        let mut path = dirs::home_dir().context("Failed to get home directory")?;
        path.push(".local/share/typy/scores.json");

        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).context("Failed to create directories")?;
            }
            File::create(&path).context("Failed to create scores.json file")?;
        }

        let file = File::open(&path).context("Failed to open scores.json file")?;
        let data: Data = match serde_json::from_reader(file) {
            Ok(data) => data,
            Err(e) if e.is_eof() => Data::default(),
            Err(e) => return Err(e).context("Failed to read scores from file"),
        };
        Ok(data)
    }

    fn write_to_file(data: Data) -> Result<()> {
        let mut path = dirs::home_dir().context("Failed to get home directory")?;
        path.push(".local/share/typy/scores.json");

        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist"));
        }

        let mut file = File::create(&path).context("Failed to truncate scores.json file")?;
        to_writer_pretty(&mut file, &data).context("Failed to write scores to file")?;

        Ok(())
    }

    pub fn get_averages() -> Result<Averages> {
        let data = Data::get_data()?;
        Ok(data.averages)
    }

    pub fn get_scores() -> Result<Vec<Score>> {
        let data = Data::get_data()?;
        Ok(data.scores)
    }

    /// Delete the score with the given timestamp and remove its contribution
    /// from the rolling averages.
    pub fn delete_score(timestamp: NaiveDateTime) -> Result<()> {
        let mut data = Data::get_data()?;
        let Some(pos) = data.scores.iter().position(|s| s.timestamp == timestamp) else {
            return Ok(());
        };
        let s = data.scores.remove(pos);
        let a = &mut data.averages;

        a.wpm_avg.count = a.wpm_avg.count.saturating_sub(1);
        a.wpm_avg.sum_all = a.wpm_avg.sum_all.saturating_sub(s.wpm);
        a.wpm_avg.avg = safe_div(a.wpm_avg.sum_all as f32, a.wpm_avg.count);

        a.raw_avg.count = a.raw_avg.count.saturating_sub(1);
        a.raw_avg.sum_all = a.raw_avg.sum_all.saturating_sub(s.raw);
        a.raw_avg.avg = safe_div(a.raw_avg.sum_all as f32, a.raw_avg.count);

        a.accuracy_avg.count = a.accuracy_avg.count.saturating_sub(1);
        a.accuracy_avg.sum_all = (a.accuracy_avg.sum_all - s.accuracy).max(0.0);
        a.accuracy_avg.avg = safe_div(a.accuracy_avg.sum_all, a.accuracy_avg.count);

        a.consistency_avg.count = a.consistency_avg.count.saturating_sub(1);
        a.consistency_avg.sum_all = (a.consistency_avg.sum_all - s.consistency).max(0.0);
        a.consistency_avg.avg = safe_div(a.consistency_avg.sum_all, a.consistency_avg.count);

        Self::write_to_file(data)?;
        Ok(())
    }
}

fn safe_div(sum: f32, count: u32) -> f32 {
    if count == 0 {
        0.0
    } else {
        sum / count as f32
    }
}

impl Default for Data {
    fn default() -> Self {
        Data {
            scores: Vec::new(),
            averages: Averages {
                wpm_avg: WpmAvg {
                    avg: 0.0,
                    count: 0,
                    sum_all: 0,
                },
                raw_avg: RawAvg {
                    avg: 0.0,
                    count: 0,
                    sum_all: 0,
                },
                accuracy_avg: AccuracyAvg {
                    avg: 0.0,
                    count: 0,
                    sum_all: 0.0,
                },
                consistency_avg: ConsistencyAvg::default(),
            },
        }
    }
}

impl Score {
    pub fn new(
        wpm: u32,
        raw: u32,
        mut accuracy: f32,
        mut consistency: f32,
        series: Vec<i32>,
        mode: String,
    ) -> Score {
        if accuracy.is_nan() {
            accuracy = 0.0;
        }
        if consistency.is_nan() {
            consistency = 0.0;
        }
        Score {
            timestamp: chrono::Local::now().naive_local(),
            wpm,
            raw,
            accuracy,
            consistency,
            series,
            mode,
        }
    }

    pub fn get_date(&self) -> String {
        self.timestamp.format("%Y-%m-%d").to_string()
    }

    pub fn get_time(&self) -> String {
        self.timestamp.format("%H:%M:%S").to_string()
    }

    pub fn sort_scores(scores: &mut [Score]) {
        scores.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    fn update_scores(score: &Score) -> Result<Vec<Score>> {
        let mut scores = Data::get_scores()?;
        scores.push(score.clone());

        if scores.len() > 10 {
            Self::sort_scores(&mut scores);
            Self::cleanup_scores(&mut scores);
        }

        Ok(scores)
    }

    fn cleanup_scores(scores: &mut Vec<Score>) {
        scores.truncate(10);
    }
}

impl Averages {
    fn new(score: Score) -> Result<Self> {
        Self::calculate_averages(score)
    }
    fn calculate_averages(score: Score) -> Result<Averages> {
        let averages = Data::get_averages()?;
        let mut wpm_sum = averages.wpm_avg.sum_all;
        let mut raw_sum = averages.raw_avg.sum_all;
        let mut accuracy_sum = averages.accuracy_avg.sum_all;
        let mut consistency_sum = averages.consistency_avg.sum_all;

        let mut wpm_count = averages.wpm_avg.count;
        let mut raw_count = averages.raw_avg.count;
        let mut accuracy_count = averages.accuracy_avg.count;
        let mut consistency_count = averages.consistency_avg.count;

        wpm_sum += score.wpm;
        raw_sum += score.raw;
        accuracy_sum += score.accuracy;
        consistency_sum += score.consistency;

        wpm_count += 1;
        raw_count += 1;
        accuracy_count += 1;
        consistency_count += 1;

        let wpm_avg = WpmAvg {
            avg: wpm_sum as f32 / wpm_count as f32,
            count: wpm_count,
            sum_all: wpm_sum,
        };

        let raw_avg = RawAvg {
            avg: raw_sum as f32 / raw_count as f32,
            count: raw_count,
            sum_all: raw_sum,
        };

        let accuracy_avg = AccuracyAvg {
            avg: accuracy_sum / accuracy_count as f32,
            count: accuracy_count,
            sum_all: accuracy_sum,
        };

        let consistency_avg = ConsistencyAvg {
            avg: consistency_sum / consistency_count as f32,
            count: consistency_count,
            sum_all: consistency_sum,
        };

        Ok(Averages {
            wpm_avg,
            raw_avg,
            accuracy_avg,
            consistency_avg,
        })
    }
}
