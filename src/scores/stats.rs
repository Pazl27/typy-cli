const AVERAGE_WORD_LENGTH: i32 = 5;

pub struct Stats {
    pub lps: Vec<i32>,
    pub letter_count: i32,
    pub incorrect_letters: i32,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            lps: Vec::new(),
            letter_count: 0,
            incorrect_letters: 0,
        }
    }

    pub fn add_letters(&mut self) {
        self.lps.push(self.letter_count);
        self.letter_count = 0;
    }

    fn total_letters(&self) -> i32 {
        self.lps.iter().sum()
    }

    fn total_seconds(&self) -> i32 {
        self.lps.len() as i32
    }

    fn minutes(&self) -> f64 {
        self.total_seconds() as f64 / 60.0
    }

    pub fn raw_wpm(&self) -> f64 {
        (self.total_letters() / AVERAGE_WORD_LENGTH) as f64 / self.minutes()
    }

    pub fn wpm(&self) -> f64 {
        const AVERAGE_WORD_LENGTH: i32 = 5;
        ((self.total_letters() - self.incorrect_letters) / AVERAGE_WORD_LENGTH) as f64
            / self.minutes()
    }

    pub fn accuracy(&self) -> f64 {
        100.0 - (self.incorrect_letters as f64 / self.total_letters() as f64) * 100.0
    }
}
