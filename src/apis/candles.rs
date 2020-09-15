use chrono::{DateTime, FixedOffset, NaiveTime};
use std::fmt;

pub type Time = NaiveTime;

pub struct Candle {
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: i64,
    datetime: DateTime<FixedOffset>,
}

impl Candle {
    pub fn new(
        open: f64,
        close: f64,
        high: f64,
        low: f64,
        volume: i64,
        datetime: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            open,
            close,
            high,
            low,
            volume,
            datetime,
        }
    }

    pub fn time(&self) -> NaiveTime {
        self.datetime.time()
    }
}

impl fmt::Display for Candle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let datetime = self.datetime.format("%D %l:%M:%S %p %z").to_string();
        write!(
            f,
            "{}, O: {}, C: {}, H: {}, L: {}, V: {}",
            datetime, self.open, self.close, self.high, self.low, self.volume
        )
    }
}
