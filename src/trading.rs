use super::clock;
use std::fmt;

const COMMISSION: f64 = 0.01; // TD Ameritrade's trade commission

pub struct Broker;

impl Broker {
    fn capital(&self) -> f64 {
        1000.0
    }

    fn buy_order(&self, _ticker: &String, _shares: i32) {}
    fn sell_order(&self, _ticker: &String, _shares: i32, _price: f64) {}
}

pub struct Account {
    pub capital: f64,
    pub pdt_remaining: i32,
    pub unsettled_cash: f64,
    pub positions: Vec<Position>,
    pub broker: Broker,
}

impl Account {
    pub fn new(broker: Broker) -> Self {
        let capital = broker.capital();
        Self {
            broker,
            capital,
            pdt_remaining: 3,
            unsettled_cash: 0.0,
            positions: Vec::new(),
        }
    }

    pub fn max_shares(&self, price: f64) -> i32 {
        (self.capital / price) as i32
    }

    pub fn open_position(&mut self, ticker: &String, bid: f64, shares: i32, time: clock::DateEST) {
        if shares <= 0 || clock::outside_market_hours(time.time()) || self.pdt_remaining < 1 {
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

        self.broker.buy_order(ticker, shares);
        self.capital -= cost;
        let pos = Position::open(shares, bid, time);
        self.positions.push(pos);
    }

    pub fn current_position(&self) -> Option<&Position> {
        self.positions.last()
    }

    pub fn close_current_position(&mut self, ticker: &String, ask: f64, time: clock::DateEST) {
        let mut position = self.positions.pop().unwrap();
        self.broker.sell_order(ticker, position.shares, ask);
        position.close(ask, time);
        self.unsettled_cash += ask * position.shares as f64;
        self.unsettled_cash -= COMMISSION;
        self.pdt_remaining -= 1;
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
        for p in &self.positions {
            if p.open {
                println!("open position {}", p.time);
            }
        }

        let mut winning_trades = Vec::new();
        let mut losing_trades = Vec::new();
        for position in &self.positions {
            if position.total_return() >= 0.0 {
                winning_trades.push(position);
            } else {
                losing_trades.push(position);
            }
        }
        winning_trades.sort_by(|a, b| b.total_return().partial_cmp(&a.total_return()).unwrap());
        losing_trades.sort_by(|a, b| a.total_return().partial_cmp(&b.total_return()).unwrap());
        let total_wins: f64 = winning_trades.iter().map(|p| p.total_return()).sum();
        let total_losses: f64 = losing_trades.iter().map(|p| p.total_return()).sum();

        let win_percent = winning_trades.len() as f64 / self.positions.len() as f64 * 100.0;
        println!(
            "W/L: {}/{} ${:.4}/${:.4} - Win: {:.2}% - Net: ${:.4}",
            winning_trades.len(),
            losing_trades.len(),
            total_wins,
            total_losses,
            win_percent,
            total_wins + total_losses,
        );

        println!("Unsettled cash: ${:.4}", self.unsettled_cash);
        println!("Ending Capital: ${:.4}", self.capital);
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

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Shares {} - Bid {} - Open? {} - Time {} - Closes {:?}",
            self.shares, self.bid, self.open, self.time, self.closes
        )
    }
}

pub struct Close {
    pub shares: i32,
    pub ask: f64,
    pub time: clock::DateEST,
}

impl fmt::Debug for Close {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Shares {} - Ask {} - Time {}",
            self.shares, self.ask, self.time
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{clock, Account, Broker, Position};
    use chrono::{DateTime, FixedOffset};

    fn datetime(h: u32, m: u32, s: u32) -> DateTime<FixedOffset> {
        let naive_datetime = clock::current_date().and_hms(h, m, s);
        DateTime::<FixedOffset>::from_utc(naive_datetime, FixedOffset::west(0))
    }

    #[test]
    fn max_shares_returns_whole_number_of_purchaseable_shares_for_price() {
        let broker = Broker {};
        let acct = Account::new(broker);
        assert_eq!(acct.max_shares(12.31), 81)
    }

    #[test]
    fn cannot_open_position_without_shares() {
        let ticker = "ABC".to_string();
        let broker = Broker {};
        let mut acct = Account::new(broker);
        acct.open_position(&ticker, 10.00, 0, datetime(9, 30, 0));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn cannot_open_position_without_enough_capital() {
        let ticker = "ABC".to_string();
        let broker = Broker {};
        let mut acct = Account::new(broker);
        acct.capital = 5.99;
        acct.open_position(&ticker, 10.00, 11, datetime(9, 31, 0));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn cannot_open_position_outside_of_market_hours() {
        let ticker = "ABC".to_string();
        let broker = Broker {};
        let mut acct = Account::new(broker);
        acct.open_position(&ticker, 10.00, 1, datetime(9, 29, 59));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn cannot_open_position_when_pdt_rule_is_hit() {
        let ticker = "ABC".to_string();
        let broker = Broker {};
        let mut acct = Account::new(broker);
        acct.pdt_remaining = 0;
        acct.open_position(&ticker, 10.00, 1, datetime(10, 29, 59));
        assert_eq!(acct.positions.len(), 0);
    }

    #[test]
    fn close_current_position_puts_return_into_unsettled_cash_minus_commission() {
        let ticker = "ABC".to_string();
        let broker = Broker {};
        let mut acct = Account::new(broker);
        acct.open_position(&ticker, 10.00, 5, datetime(9, 30, 0));
        acct.close_current_position(&ticker, 11.00, datetime(9, 31, 0));
        assert_eq!(acct.unsettled_cash, 54.99);
        assert_eq!(acct.capital, 950.00);
    }

    #[test]
    fn position_provides_return_value() {
        let mut position = Position::open(10, 5.00, datetime(9, 31, 0));
        position.close(6.00, datetime(9, 31, 0));
        let total_return = position.total_return();
        assert_eq!(total_return, 10.00);
    }
}
