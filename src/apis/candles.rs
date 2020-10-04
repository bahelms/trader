use crate::clock;
use std::fmt;

pub struct CandlesIter<'a, I> {
    pub previous_candle: Option<&'a Candle>,
    list: I,
}

impl<'a, I> Iterator for CandlesIter<'a, I>
where
    I: Iterator<Item = &'a Candle>,
{
    type Item = &'a Candle;

    fn next(&mut self) -> Option<Self::Item> {
        self.previous_candle = self.list.next();
        self.previous_candle
    }
}

pub struct Candle {
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: i64,
    pub datetime: clock::DateEST,
}

impl Candle {
    pub fn new(
        open: f64,
        close: f64,
        high: f64,
        low: f64,
        volume: i64,
        datetime: clock::DateEST,
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

    pub fn is_bull(&self) -> bool {
        self.close > self.open
    }

    pub fn is_bear(&self) -> bool {
        self.close < self.open
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
