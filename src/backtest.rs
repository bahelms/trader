use super::{
    apis::td_ameritrade,
    clock, strategies,
    trading::{Position, PricePeriod, Trades},
};

pub fn backtest(symbol: String, mut client: td_ameritrade::Client, capital: f64) {
    let price_period = PricePeriod::new("1", "day", "1", "minute");
    let mut trades = Backtest::new(capital, price_period);

    let candles = client.price_history(&symbol, &trades.price_period);
    if candles.len() < 9 {
        eprintln!("Not enough candles for minimum trading: {}", candles.len());
        return;
    }
    for c in &candles {
        println!("candle: {}", c);
    }
    println!("\nSMA 9");
    trades = strategies::sma_crossover(&candles, trades, 9);
    trades.log_results();

    println!("\nSMA 9");
    let price_period = PricePeriod::new("1", "day", "5", "minute");
    trades = Backtest::new(capital, price_period);
    let candles = client.price_history(&symbol, &trades.price_period);
    if candles.len() < 9 {
        eprintln!("Not enough candles for minimum trading: {}", candles.len());
        return;
    }
    trades = strategies::sma_crossover(&candles, trades, 9);
    trades.log_results();
}

pub struct Backtest {
    pub positions: Vec<Position>,
    pub capital: f64,
    pub price_period: PricePeriod,
}

impl Backtest {
    pub fn new(capital: f64, price_period: PricePeriod) -> Self {
        Self {
            capital,
            price_period,
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

        println!("{}\n", self.price_period);
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
            let close_time = pos.closes[0].time.time();
            println!(
                "\t* open: {} - close: {}, return: ${}",
                pos.time,
                close_time,
                pos.total_return()
            );
        }
        stop = 5;
        if losing_trades.len() < stop {
            stop = losing_trades.len();
        }
        println!("\nTop {} Losers", stop);
        for pos in &losing_trades[..stop] {
            let close_time = pos.closes[0].time.time();
            println!(
                "\t* open: {} - close: {}, return: ${}",
                pos.time,
                close_time,
                pos.total_return()
            );
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

    fn close_current_position(&mut self, ask: f64, time: clock::DateEST) {
        let mut position = self.positions.pop().unwrap();
        position.close(ask, time);
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

#[cfg(test)]
mod tests {
    use super::{Backtest, Position, Trades};

    #[test]
    fn cannot_open_position_without_shares() {
        let mut trades = Backtest::new(100.00);
        trades.open_position(10.00, 0.0);
        assert_eq!(trades.positions.len(), 0);
    }
}
