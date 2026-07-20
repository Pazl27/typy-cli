const AVERAGE_WORD_LENGTH: f64 = 5.0;

pub struct Stats {
    pub lps: Vec<i32>,
    pub letter_count: i32,
    correct: i32,
    incorrect: i32,
    extra: i32,
    missed: i32,
    spaces: i32,
    elapsed_secs: f64,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            lps: Vec::new(),
            letter_count: 0,
            correct: 0,
            incorrect: 0,
            extra: 0,
            missed: 0,
            spaces: 0,
            elapsed_secs: 0.0,
        }
    }

    pub fn add_letters(&mut self) {
        self.lps.push(self.letter_count);
        self.letter_count = 0;
    }

    pub fn finalize(
        &mut self,
        correct: i32,
        incorrect: i32,
        extra: i32,
        missed: i32,
        spaces: i32,
        elapsed_secs: f64,
    ) {
        self.correct = correct;
        self.incorrect = incorrect;
        self.extra = extra;
        self.missed = missed;
        self.spaces = spaces;
        self.elapsed_secs = elapsed_secs;
    }

    fn minutes(&self) -> f64 {
        self.elapsed_secs / 60.0
    }

    fn words_per(&self, chars: f64) -> f64 {
        let minutes = self.minutes();
        if minutes <= 0.0 {
            0.0
        } else {
            (chars / AVERAGE_WORD_LENGTH) / minutes
        }
    }

    pub fn raw_wpm(&self) -> f64 {
        let typed = self.correct + self.incorrect + self.extra + self.spaces;
        self.words_per(typed as f64)
    }

    pub fn wpm(&self) -> f64 {
        let net = self.correct + self.spaces;
        self.words_per(net as f64)
    }

    pub fn accuracy(&self) -> f64 {
        let total = self.correct + self.incorrect + self.extra + self.missed;
        if total == 0 {
            0.0
        } else {
            self.correct as f64 / total as f64 * 100.0
        }
    }
}

#[cfg(test)]
mod stats_tests {
    use super::*;

    fn finalized(
        correct: i32,
        incorrect: i32,
        extra: i32,
        missed: i32,
        spaces: i32,
        elapsed: f64,
    ) -> Stats {
        let mut s = Stats::new();
        s.finalize(correct, incorrect, extra, missed, spaces, elapsed);
        s
    }

    #[test]
    fn perfect_run_counts_spaces() {
        let s = finalized(150, 0, 0, 0, 30, 30.0);
        assert_eq!(s.wpm().round() as i32, 72);
        assert_eq!(s.raw_wpm().round() as i32, 72);
        assert!((s.accuracy() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn errors_lower_net_and_accuracy_not_raw() {
        let s = finalized(140, 10, 0, 0, 30, 30.0);
        assert_eq!(s.wpm().round() as i32, 68);
        assert_eq!(s.raw_wpm().round() as i32, 72);
        assert!((s.accuracy() - 93.3333).abs() < 0.01);
    }

    #[test]
    fn missed_chars_lower_accuracy() {
        let s = finalized(100, 0, 0, 20, 20, 30.0);
        assert!((s.accuracy() - 83.3333).abs() < 0.01);
    }

    #[test]
    fn no_integer_truncation() {
        let s = finalized(149, 0, 0, 0, 0, 60.0);
        assert!((s.wpm() - 29.8).abs() < 1e-9);
    }

    #[test]
    fn zero_time_and_empty_are_safe() {
        let empty = Stats::new();
        assert_eq!(empty.wpm(), 0.0);
        assert_eq!(empty.raw_wpm(), 0.0);
        assert_eq!(empty.accuracy(), 0.0);

        let zero_time = finalized(10, 0, 0, 0, 2, 0.0);
        assert_eq!(zero_time.wpm(), 0.0);
        assert_eq!(zero_time.accuracy(), 100.0);
    }
}
