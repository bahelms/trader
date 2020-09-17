use super::clock;
use std::fmt;

pub trait Trades {
    fn open_position(&mut self, bid: f64, shares: f64, time: clock::DateEST);
    fn close_current_position(&mut self, ask: f64);
    fn max_purchaseable_shares(&self, price: f64) -> f64;
    fn current_position(&self) -> Option<&Position>;

    fn is_current_position_open(&self) -> bool {
        if let Some(position) = self.current_position() {
            position.open
        } else {
            false
        }
    }
}

pub struct Live {
    pub positions: Vec<Position>,
    pub capital: f64,
    pub pdt_remaining: i32,
    pub unsettled_cash: f64,
}

pub struct Position {
    pub open: bool,
    pub shares: f64,
    pub bid: f64,
    pub closes: Vec<Close>,
    pub time: clock::DateEST,
}

pub struct Close {
    pub shares: f64,
    pub ask: f64,
    pub time: clock::DateEST,
}

pub struct PricePeriod {
    pub period: &'static str,
    pub period_type: &'static str,
    pub frequency: &'static str,
    pub frequency_type: &'static str,
}

#[allow(dead_code)]
impl Live {
    pub fn new(capital: f64) -> Self {
        Self {
            capital,
            positions: Vec::new(),
            pdt_remaining: 3,
            unsettled_cash: 0.0,
        }
    }
}

impl Trades for Live {
    fn current_position(&self) -> Option<&Position> {
        self.positions.last()
    }

    fn open_position(&mut self, bid: f64, shares: f64, _time: clock::DateEST) {
        if shares <= 0.0 {
            return;
        }
        let cost = bid * shares;
        if cost > self.capital {
            println!(
                "Not enough capital for position - capital: {}, cost: {}",
                self.capital, cost
            );
            return;
        }

        // send buy order
        self.capital -= cost;
        let pos = Position::open(shares, bid, clock::current_datetime());
        self.positions.push(pos);
    }

    fn close_current_position(&mut self, ask: f64) {
        // send sell order
        let mut position = self.positions.pop().unwrap();
        position.close(ask);
        self.unsettled_cash += ask * position.shares;
        self.positions.push(position);
    }

    fn max_purchaseable_shares(&self, price: f64) -> f64 {
        self.capital / price
    }
}

impl Position {
    pub fn open(shares: f64, bid: f64, time: clock::DateEST) -> Self {
        Self {
            shares,
            bid,
            time,
            open: true,
            closes: Vec::new(),
        }
    }

    pub fn close(&mut self, ask: f64) {
        self.open = false;
        self.closes = vec![Close {
            ask,
            shares: self.shares,
            time: clock::current_datetime(),
        }];
    }

    pub fn total_return(&self) -> f64 {
        let gross: f64 = self
            .closes
            .iter()
            .map(|close| close.ask * close.shares)
            .sum();
        gross - self.bid * self.shares
    }
}

impl PricePeriod {
    pub fn new(
        period: &'static str,
        period_type: &'static str,
        frequency: &'static str,
        frequency_type: &'static str,
    ) -> Self {
        Self {
            period,
            period_type,
            frequency,
            frequency_type,
        }
    }
}

impl fmt::Display for PricePeriod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} - {} {}",
            self.period, self.period_type, self.frequency, self.frequency_type,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Live, Position, Trades};

    #[test]
    fn close_current_position_puts_return_into_unsettled_cash() {
        let mut trades = Live::new(100.00);
        trades.open_position(10.00, 5.0);
        trades.close_current_position(11.00);
        assert_eq!(trades.unsettled_cash, 55.00);
        assert_eq!(trades.capital, 50.00);
    }

    #[test]
    fn cannot_open_position_without_enough_capital() {
        let mut trades = Live::new(100.00);
        trades.open_position(10.00, 11.0);
        assert_eq!(trades.positions.len(), 0);
    }

    #[test]
    fn cannot_open_position_without_shares() {
        let mut trades = Live::new(100.00);
        trades.open_position(10.00, 0.0);
        assert_eq!(trades.positions.len(), 0);
    }

    #[test]
    fn position_provides_return_value() {
        let mut position = Position::open(10.0, 5.00);
        position.close(6.00);
        let total_return = position.total_return();
        assert_eq!(total_return, 10.00);
    }
}
