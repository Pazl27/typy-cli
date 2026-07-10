//! Typing statistics derived from the raw keystroke log.
//!
//! Everything is computed from the `Keystroke` history, which records *every*
//! key press with its timestamp and correctness (corrected mistakes stay in the
//! log). This gives real per-second timing for the WPM curve, a consistency
//! score, and a per-character error breakdown.

use std::collections::BTreeMap;

use crate::terminal::Keystroke;

const AVERAGE_WORD_LENGTH: f64 = 5.0;

pub struct Stats {
    total_keys: usize,
    correct_keys: usize,
    duration_ms: u128,
    /// Total characters typed in each 1-second bucket.
    per_second_total: Vec<u32>,
    /// (attempts, errors) per expected character.
    errors_by_char: BTreeMap<char, (u32, u32)>,
}

impl Stats {
    /// Build stats from the keystroke log. `elapsed_ms` is the total test
    /// duration (for time mode this is the configured time; for word mode it is
    /// the actual time taken).
    pub fn from_keys(keys: &[Keystroke], elapsed_ms: u128) -> Self {
        let total_keys = keys.len();
        let correct_keys = keys.iter().filter(|k| k.correct).count();

        let n_buckets = (elapsed_ms / 1000) as usize + 1;
        let mut per_second_total = vec![0u32; n_buckets.max(1)];
        let mut errors_by_char: BTreeMap<char, (u32, u32)> = BTreeMap::new();

        for k in keys {
            let bucket = ((k.t_ms / 1000) as usize).min(per_second_total.len() - 1);
            per_second_total[bucket] += 1;
            if let Some(tc) = k.target {
                let entry = errors_by_char.entry(tc).or_insert((0, 0));
                entry.0 += 1;
                if !k.correct {
                    entry.1 += 1;
                }
            }
        }

        Stats {
            total_keys,
            correct_keys,
            duration_ms: elapsed_ms.max(1),
            per_second_total,
            errors_by_char,
        }
    }

    fn minutes(&self) -> f64 {
        self.duration_ms as f64 / 60_000.0
    }

    /// Net WPM: only correctly typed characters count.
    pub fn wpm(&self) -> f64 {
        if self.minutes() == 0.0 {
            return 0.0;
        }
        (self.correct_keys as f64 / AVERAGE_WORD_LENGTH) / self.minutes()
    }

    /// Raw WPM: every keystroke counts, right or wrong.
    pub fn raw_wpm(&self) -> f64 {
        if self.minutes() == 0.0 {
            return 0.0;
        }
        (self.total_keys as f64 / AVERAGE_WORD_LENGTH) / self.minutes()
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_keys == 0 {
            return 0.0;
        }
        self.correct_keys as f64 / self.total_keys as f64 * 100.0
    }

    /// Consistency = how steady the per-second speed was, as a percentage.
    /// Computed as `1 - (stddev / mean)` over the per-second WPM series.
    pub fn consistency(&self) -> f64 {
        let series: Vec<f64> = self
            .per_second_total
            .iter()
            .map(|&c| c as f64 / AVERAGE_WORD_LENGTH * 60.0)
            .collect();
        if series.len() < 2 {
            return 100.0;
        }
        let mean = series.iter().sum::<f64>() / series.len() as f64;
        if mean == 0.0 {
            return 0.0;
        }
        let variance =
            series.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / series.len() as f64;
        let cv = variance.sqrt() / mean;
        ((1.0 - cv) * 100.0).clamp(0.0, 100.0)
    }

    /// Per-second WPM series, for the finish graph.
    pub fn wpm_series(&self) -> Vec<i32> {
        self.per_second_total
            .iter()
            .map(|&c| (c as f64 / AVERAGE_WORD_LENGTH * 60.0).round() as i32)
            .collect()
    }

    /// The `n` most-missed characters as `(char, errors, attempts)`, ordered by
    /// error count then error rate.
    pub fn top_errors(&self, n: usize) -> Vec<(char, u32, u32)> {
        let mut v: Vec<(char, u32, u32)> = self
            .errors_by_char
            .iter()
            .filter(|(_, (_, errs))| *errs > 0)
            .map(|(&c, &(att, errs))| (c, errs, att))
            .collect();
        v.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| (b.1 as f64 / b.2 as f64).partial_cmp(&(a.1 as f64 / a.2 as f64)).unwrap())
        });
        v.truncate(n);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ks(t_ms: u128, target: char, typed: char) -> Keystroke {
        Keystroke {
            t_ms,
            target: Some(target),
            typed,
            correct: target == typed,
        }
    }

    #[test]
    fn accuracy_counts_correct_over_total() {
        let keys = vec![ks(0, 'a', 'a'), ks(100, 'b', 'x'), ks(200, 'c', 'c')];
        let s = Stats::from_keys(&keys, 1000);
        assert!((s.accuracy() - 66.6666).abs() < 0.1);
    }

    #[test]
    fn wpm_uses_correct_only_raw_uses_all() {
        // 10 keys over 60s, 5 correct -> net 1 wpm, raw 2 wpm
        let mut keys = Vec::new();
        for i in 0..10u128 {
            let (t, ty) = if i < 5 { ('a', 'a') } else { ('a', 'x') };
            keys.push(ks(i * 10, t, ty));
        }
        let s = Stats::from_keys(&keys, 60_000);
        assert!((s.wpm() - 1.0).abs() < 0.001);
        assert!((s.raw_wpm() - 2.0).abs() < 0.001);
    }

    #[test]
    fn top_errors_ranks_most_missed() {
        let keys = vec![
            ks(0, 'e', 'x'),
            ks(1, 'e', 'x'),
            ks(2, 't', 'y'),
            ks(3, 'a', 'a'),
        ];
        let s = Stats::from_keys(&keys, 1000);
        let top = s.top_errors(5);
        assert_eq!(top[0].0, 'e');
        assert_eq!(top[0].1, 2); // 2 errors
    }
}
