use super::{apis, apis::candles::Candle, clock};
use std::fmt;

pub struct PriceData<'a> {
    client: apis::polygon::Client<'a>,
    candles: Vec<Candle>,
    current_index: usize,
}

impl<'a> PriceData<'a> {
    pub fn new(client: apis::polygon::Client<'a>) -> Self {
        Self {
            client,
            candles: Vec::new(),
            current_index: 0,
        }
    }

    pub fn history(&mut self, ticker: &String, bars: usize, frequency: &str) -> Option<&[Candle]> {
        let (frequency, frequency_type) = parse_frequency(frequency);
        let start_date = clock::days_ago(60);
        let end_date = clock::current_date();
        let history =
            self.client
                .price_history(ticker, start_date, end_date, frequency, frequency_type);

        if let Some(candles) = history {
            self.candles = candles;
            self.current_index = bars;
            Some(&self.candles[..bars])
        } else {
            None
        }
    }

    pub fn next_candle(&mut self) -> Option<&Candle> {
        let candle = self.candles.get(self.current_index);
        self.current_index += 1;
        candle
    }
}

pub trait Broker {
    fn capital(&mut self, time: clock::LocalDateTime) -> f64;
    fn unsettled_cash(&self) -> f64;
    fn is_market_open(&self, datetime: clock::LocalDateTime) -> bool;
    fn sell_order(&mut self, _ticker: &String, shares: i32, price: f64, time: clock::LocalDateTime);
    fn buy_order(
        &mut self,
        _ticker: &String,
        shares: i32,
        price: f64,
        _time: clock::LocalDateTime,
    ) -> Option<f64>;
}

pub struct Account<'a, B> {
    pub positions: Vec<Position<'a>>,
    pub broker: B,
}

impl<'a, B> Account<'a, B>
where
    B: Broker,
{
    pub fn new(broker: B) -> Self {
        Self {
            broker,
            positions: Vec::new(),
        }
    }

    pub fn total_cash(&mut self, time: clock::LocalDateTime) -> f64 {
        self.broker.unsettled_cash() + self.broker.capital(time)
    }

    pub fn max_shares(&mut self, price: f64, time: clock::LocalDateTime) -> i32 {
        (self.broker.capital(time) / price) as i32
    }

    pub fn open_position(
        &mut self,
        ticker: &'a String,
        bid: f64,
        shares: i32,
        time: clock::LocalDateTime,
    ) {
        if shares <= 0 || !self.broker.is_market_open(time) {
            return;
        }

        if let Some(_) = self.broker.buy_order(ticker, shares, bid, time) {
            let pos = Position::open(ticker, shares, bid, time);
            self.positions.push(pos);
        }
    }

    pub fn current_position(&self) -> Option<&Position> {
        self.positions.last()
    }

    pub fn close_position(&mut self, ticker: &String, ask: f64, time: clock::LocalDateTime) {
        let mut position = self.positions.pop().unwrap();
        self.broker.sell_order(ticker, position.shares, ask, time);
        position.close(ask, time);
        self.positions.push(position);
    }

    pub fn is_position_open(&self) -> bool {
        if let Some(position) = self.current_position() {
            position.open
        } else {
            false
        }
    }

    pub fn close_position_for_day(&mut self, ticker: &String, candle: &Candle) {
        let close_time = clock::Time::from_hms(15, 55, 0);
        if self.is_position_open() && candle.datetime.time() >= close_time {
            self.close_position(ticker, candle.close, candle.datetime);
        }
    }
}

pub struct Position<'a> {
    pub open: bool,
    pub shares: i32,
    pub bid: f64,
    pub closes: Vec<Close>,
    pub time: clock::LocalDateTime,
    pub ticker: &'a String,
}

impl<'a> Position<'a> {
    pub fn open(ticker: &'a String, shares: i32, bid: f64, time: clock::LocalDateTime) -> Self {
        Self {
            shares,
            bid,
            time,
            ticker,
            open: true,
            closes: Vec::new(),
        }
    }

    pub fn close(&mut self, ask: f64, time: clock::LocalDateTime) {
        self.open = false;
        self.closes = vec![Close {
            ask,
            time,
            shares: self.shares,
        }];
    }

    pub fn total_return(&self) -> f64 {
        let gross: f64 = self
            .closes
            .iter()
            .map(|close| close.ask * close.shares as f64)
            .sum();
        gross - self.bid * self.shares as f64
    }
}

impl<'a> fmt::Display for Position<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} @ ${:<9} - {} - Closed {:?} -- return ${:.2}",
            self.ticker,
            self.shares,
            self.bid,
            self.time,
            self.closes,
            self.total_return()
        )
    }
}

pub struct Close {
    pub shares: i32,
    pub ask: f64,
    pub time: clock::LocalDateTime,
}

impl fmt::Debug for Close {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ ${} - {}", self.shares, self.ask, self.time)
    }
}

fn parse_frequency(code: &str) -> (String, String) {
    let strings: Vec<&str> = code.split(':').collect();
    (strings[0].to_string(), strings[1].to_string())
}
