use super::{apis, apis::candles::Candle, clock};
use std::fmt;

const COMMISSION: f64 = 0.01; // TD Ameritrade's trade commission

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

pub struct SimBroker {
    capital: f64,
    unsettled_cash: f64,
    settle_date: Option<clock::LocalDate>,
}

impl SimBroker {
    pub fn new() -> Self {
        Self {
            capital: 1000.0,
            unsettled_cash: 0.0,
            settle_date: None,
        }
    }
}

impl Broker for SimBroker {
    fn capital(&mut self, time: clock::LocalDateTime) -> f64 {
        if let Some(settle_date) = self.settle_date {
            if time.date() >= settle_date {
                self.capital += self.unsettled_cash;
                self.unsettled_cash = 0.0;
            }
        }
        self.capital
    }

    fn unsettled_cash(&self) -> f64 {
        self.unsettled_cash
    }

    fn is_market_open(&self, datetime: clock::LocalDateTime) -> bool {
        let open = clock::Time::from_hms(9, 30, 0);
        let close = clock::Time::from_hms(16, 0, 0);
        let time = datetime.time();
        let day_of_week: i32 = datetime.date().format("%u").to_string().parse().unwrap();
        time >= open && time < close && day_of_week < 6
    }

    fn buy_order(
        &mut self,
        _ticker: &String,
        shares: i32,
        price: f64,
        _time: clock::LocalDateTime,
    ) -> Option<f64> {
        if self.unsettled_cash > 0.0 {
            return None;
        }

        let cost = price * shares as f64;
        if cost > self.capital {
            return None;
        }

        self.capital -= cost;
        Some(self.capital)
    }

    fn sell_order(
        &mut self,
        _ticker: &String,
        shares: i32,
        price: f64,
        time: clock::LocalDateTime,
    ) {
        self.unsettled_cash = (price * shares as f64) - COMMISSION;
        let mut settle_date = time.date() + clock::days(2);
        while clock::day_of_week(settle_date) > 5 {
            settle_date = settle_date + clock::days(1);
        }
        self.settle_date = Some(settle_date);
    }
}

pub struct Account<'a, B> {
    pub positions: Vec<Position<'a>>,
    broker: B,
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

    pub fn final_capital(&mut self, time: clock::LocalDateTime) -> f64 {
        self.broker.unsettled_cash() + self.broker.capital(time)
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
            "{}: {} @ {} - {} - Closed {:?} -- return ${:.2}",
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

#[cfg(test)]
mod tests {
    use super::{apis::candles::Candle, clock, Account, Broker, Position, SimBroker};

    #[test]
    fn max_shares_returns_whole_number_of_purchaseable_shares_for_price() {
        let mut acct = Account::new(SimBroker::new());
        assert_eq!(
            acct.max_shares(12.31, clock::datetime(2020, 9, 29, 9, 30, 0)),
            81
        );
    }

    #[test]
    fn cannot_open_position_without_shares() {
        let ticker = "ABC".to_string();
        let mut acct = Account::new(SimBroker::new());
        acct.open_position(&ticker, 10.00, 0, clock::datetime(2020, 9, 29, 9, 30, 0));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn cannot_open_position_without_enough_capital() {
        let ticker = "ABC".to_string();
        let broker = SimBroker {
            capital: 5.99,
            unsettled_cash: 0.0,
            settle_date: None,
        };
        let mut acct = Account::new(broker);
        acct.open_position(&ticker, 10.00, 11, clock::datetime(2020, 9, 29, 9, 31, 0));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn cannot_open_position_outside_of_market_hours() {
        let ticker = "ABC".to_string();
        let mut acct = Account::new(SimBroker::new());
        acct.open_position(&ticker, 10.00, 1, clock::datetime(2020, 9, 29, 9, 29, 59));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn close_position_puts_return_into_unsettled_cash_minus_commission() {
        let date = clock::datetime(2020, 9, 29, 9, 30, 0);
        let ticker = "ABC".to_string();
        let mut acct = Account::new(SimBroker::new());
        acct.open_position(&ticker, 10.00, 5, date);
        acct.close_position(&ticker, 11.00, clock::datetime(2020, 9, 29, 9, 31, 0));
        assert_eq!(acct.broker.unsettled_cash(), 54.99);
        assert_eq!(
            acct.broker.capital(clock::datetime(2020, 9, 29, 9, 30, 0)),
            950.00
        );
    }

    #[test]
    fn position_provides_return_value() {
        let ticker = "ABC".to_string();
        let mut position =
            Position::open(&ticker, 10, 5.00, clock::datetime(2020, 9, 29, 9, 31, 0));
        position.close(6.00, clock::datetime(2020, 9, 29, 9, 31, 0));
        let total_return = position.total_return();
        assert_eq!(total_return, 10.00);
    }

    #[test]
    fn market_hours() {
        let broker = SimBroker::new();
        let time = clock::datetime(2020, 9, 29, 9, 30, 00);
        assert_eq!(broker.is_market_open(time), true);
    }

    #[test]
    fn pre_market_hours() {
        let broker = SimBroker::new();
        let time = clock::datetime(2020, 9, 29, 9, 29, 59);
        assert_eq!(broker.is_market_open(time), false);
    }

    #[test]
    fn post_market_hours() {
        let broker = SimBroker::new();
        let time = clock::datetime(2020, 9, 29, 16, 0, 0);
        assert_eq!(broker.is_market_open(time), false);
    }

    #[test]
    fn saturday_is_outside_market_hours() {
        let broker = SimBroker::new();
        let time = clock::datetime(2020, 9, 26, 10, 0, 0);
        assert_eq!(broker.is_market_open(time), false);
    }

    #[test]
    fn sunday_is_outside_market_hours() {
        let broker = SimBroker::new();
        let time = clock::datetime(2020, 9, 27, 10, 0, 0);
        assert_eq!(broker.is_market_open(time), false);
    }

    #[test]
    fn closing_position_on_friday_puts_settle_date_on_monday() {
        let ticker = "ABC".to_string();
        let mut acct = Account::new(SimBroker::new());
        let close_time = clock::datetime(2020, 9, 25, 10, 0, 1);

        acct.open_position(&ticker, 100.00, 10, clock::datetime(2020, 9, 25, 10, 0, 0));
        acct.close_position(&ticker, 100.00, close_time);
        assert_eq!(acct.broker.capital(close_time), 0.0);
        assert_eq!(acct.broker.capital(close_time + clock::days(1)), 0.0);
        assert_eq!(acct.broker.capital(close_time + clock::days(2)), 0.0);
        assert_eq!(acct.broker.capital(close_time + clock::days(3)), 999.99);
    }

    #[test]
    fn closing_position_on_thursday_puts_settle_date_on_monday() {
        let ticker = "ABC".to_string();
        let mut acct = Account::new(SimBroker::new());
        let close_time = clock::datetime(2020, 9, 24, 10, 0, 1);

        acct.open_position(&ticker, 100.00, 10, clock::datetime(2020, 9, 24, 10, 0, 0));
        acct.close_position(&ticker, 100.00, close_time);
        assert_eq!(acct.broker.capital(close_time), 0.0);
        assert_eq!(acct.broker.capital(close_time + clock::days(1)), 0.0);
        assert_eq!(acct.broker.capital(close_time + clock::days(2)), 0.0);
        assert_eq!(acct.broker.capital(close_time + clock::days(3)), 0.0);
        assert_eq!(acct.broker.capital(close_time + clock::days(4)), 999.99);
    }

    #[test]
    fn account_will_close_open_position_within_five_minutes_of_market_close() {
        let ticker = "ABC".to_string();
        let mut acct = Account::new(SimBroker::new());
        let candle_time = clock::datetime(2020, 9, 24, 15, 55, 00);
        let candle = Candle::new(0.0, 101.0, 0.0, 0.0, 0, candle_time);

        acct.open_position(&ticker, 100.00, 10, clock::datetime(2020, 9, 24, 10, 0, 0));
        acct.close_position_for_day(&ticker, &candle);
        assert_eq!(acct.positions[0].open, false);
    }
}
