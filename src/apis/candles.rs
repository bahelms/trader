use std::fmt;

pub struct Candle {
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: i64,
    time: String,
}

impl Candle {
    pub fn new(open: f64, close: f64, high: f64, low: f64, volume: i64, time: String) -> Self {
        Self {
            open,
            close,
            high,
            low,
            volume,
            time,
        }
    }
}

impl fmt::Display for Candle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, O: {}, C: {}, H: {}, L: {}, V: {}",
            self.time, self.open, self.close, self.high, self.low, self.volume
        )
    }
}
