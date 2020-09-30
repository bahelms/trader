use super::clock;
// use std::fmt;

pub struct Account {
    pub capital: f64,
    pub pdt_remaining: i32,
    pub unsettled_cash: f64,
    pub positions: Vec<Position>,
}

impl Account {
    pub fn new(capital: f64) -> Self {
        Self {
            capital,
            pdt_remaining: 3,
            unsettled_cash: 0.0,
            positions: Vec::new(),
        }
    }

    pub fn max_shares(&self, price: f64) -> i32 {
        (self.capital / price) as i32
    }

    pub fn open_position(&mut self, bid: f64, shares: i32, time: clock::DateEST) {
        if shares <= 0 || clock::outside_market_hours(time.time()) {
            return;
        }

        let cost = bid * shares as f64;
        if cost > self.capital {
            println!(
                "Not enough capital for position - capital: {}, cost: {}",
                self.capital, cost
            );
            return;
        }

        // send buy order
        self.capital -= cost;
        let pos = Position::open(shares, bid, time);
        self.positions.push(pos);
    }

    pub fn current_position(&self) -> Option<&Position> {
        self.positions.last()
    }

    pub fn close_current_position(&mut self, ask: f64, time: clock::DateEST) {
        // send sell order
        let mut position = self.positions.pop().unwrap();
        position.close(ask, time);
        self.unsettled_cash += ask * position.shares as f64;
        self.positions.push(position);
    }

    pub fn is_current_position_open(&self) -> bool {
        if let Some(position) = self.current_position() {
            position.open
        } else {
            false
        }
    }

    pub fn log_results(&self) {
        println!("log results");
    }
}

pub struct Position {
    pub open: bool,
    pub shares: i32,
    pub bid: f64,
    pub closes: Vec<Close>,
    pub time: clock::DateEST,
}

impl Position {
    pub fn open(shares: i32, bid: f64, time: clock::DateEST) -> Self {
        Self {
            shares,
            bid,
            time,
            open: true,
            closes: Vec::new(),
        }
    }

    pub fn close(&mut self, ask: f64, time: clock::DateEST) {
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

pub struct Close {
    pub shares: i32,
    pub ask: f64,
    pub time: clock::DateEST,
}

#[cfg(test)]
mod tests {
    use super::{clock, Account, Position};
    use chrono::{DateTime, FixedOffset};

    fn datetime(h: u32, m: u32, s: u32) -> DateTime<FixedOffset> {
        let naive_datetime = clock::current_date().and_hms(h, m, s);
        DateTime::<FixedOffset>::from_utc(naive_datetime, FixedOffset::west(0))
    }

    #[test]
    fn max_shares_returns_whole_number_of_purchaseable_shares_for_price() {
        let acct = Account::new(555.7);
        assert_eq!(acct.max_shares(12.31), 45)
    }

    #[test]
    fn cannot_open_position_without_shares() {
        let mut acct = Account::new(100.0);
        acct.open_position(10.00, 0, datetime(9, 30, 0));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn cannot_open_position_without_enough_capital() {
        let mut acct = Account::new(100.0);
        acct.open_position(10.00, 11, clock::current_datetime());
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn cannot_open_position_outside_of_market_hours() {
        let mut acct = Account::new(100.0);
        acct.open_position(10.00, 1, datetime(9, 29, 59));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn close_current_position_puts_return_into_unsettled_cash() {
        let mut acct = Account::new(100.0);
        acct.open_position(10.00, 5, datetime(9, 30, 0));
        acct.close_current_position(11.00, clock::current_datetime());
        assert_eq!(acct.unsettled_cash, 55.00);
        assert_eq!(acct.capital, 50.00);
    }

    #[test]
    fn position_provides_return_value() {
        let mut position = Position::open(10, 5.00, clock::current_datetime());
        position.close(6.00, clock::current_datetime());
        let total_return = position.total_return();
        assert_eq!(total_return, 10.00);
    }
}
