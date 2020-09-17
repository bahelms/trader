use super::clock;

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

pub struct Backtest {
    pub positions: Vec<Position>,
    pub capital: f64,
    pub chart_period: &'static str,
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

impl Backtest {
    pub fn new(capital: f64, chart_period: &'static str) -> Self {
        Self {
            capital,
            chart_period,
            positions: Vec::new(),
        }
    }

    pub fn log_results(&self) {
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

        println!("{}\n", self.chart_period);
        println!(
            "Trades won: {} - total returns: ${}",
            winning_trades.len(),
            total_wins
        );
        println!(
            "Trades lost: {} - total returns: ${}",
            losing_trades.len(),
            total_losses
        );
        println!(
            "Win %: {:.2}",
            winning_trades.len() as f64 / self.positions.len() as f64 * 100.0
        );

        println!("Net return: ${}", total_wins + total_losses);
        println!("Ending Capital: ${}", self.capital);

        let mut stop = 5;
        if winning_trades.len() < stop {
            stop = winning_trades.len();
        }
        println!("\nTop {} Winners", stop);
        for pos in &winning_trades[..stop] {
            println!("\t* time: {}, return: ${}", pos.time, pos.total_return());
        }
        stop = 5;
        if losing_trades.len() < stop {
            stop = losing_trades.len();
        }
        println!("\nTop {} Losers", stop);
        for pos in &losing_trades[..stop] {
            println!("\t* time: {}, return: ${}", pos.time, pos.total_return());
        }
    }
}

impl Trades for Backtest {
    fn open_position(&mut self, bid: f64, shares: f64, time: clock::DateEST) {
        if shares <= 0.0 {
            return;
        }
        self.capital -= bid * shares;
        self.positions.push(Position::open(shares, bid, time));
    }

    fn close_current_position(&mut self, ask: f64) {
        let mut position = self.positions.pop().unwrap();
        position.close(ask);
        self.capital += ask * position.shares;
        self.positions.push(position);
    }

    fn max_purchaseable_shares(&self, price: f64) -> f64 {
        self.capital / price
    }

    fn current_position(&self) -> Option<&Position> {
        self.positions.last()
    }
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

#[cfg(test)]
mod tests {
    use super::{Backtest, Live, Position, Trades};

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

        let mut trades = Backtest::new(100.00);
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
