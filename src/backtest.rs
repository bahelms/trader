use super::{
    apis::polygon,
    clock, config, strategies,
    trading::{Account, Broker, PriceData},
};

pub struct BacktestBroker {
    capital: f64,
}

impl Broker for BacktestBroker {
    fn capital(&mut self, _time: clock::LocalDateTime) -> f64 {
        self.capital
    }

    fn unsettled_cash(&self) -> f64 {
        0.0
    }

    fn is_market_open(&self, datetime: clock::LocalDateTime) -> bool {
        let open = clock::Time::from_hms(9, 30, 0);
        let close = clock::Time::from_hms(16, 0, 0);
        let time = datetime.time();
        let day_of_week: i32 = datetime.date().format("%u").to_string().parse().unwrap();
        time >= open && time < close && day_of_week < 6
    }

    fn sell_order(
        &mut self,
        _ticker: &String,
        _shares: i32,
        _price: f64,
        _time: clock::LocalDateTime,
    ) {
    }

    fn buy_order(
        &mut self,
        _ticker: &String,
        _shares: i32,
        _price: f64,
        _time: clock::LocalDateTime,
    ) -> Option<f64> {
        Some(self.capital)
    }
}

pub fn run_backtest(tickers: &[String], env: &config::Env) {
    println!("Backtesting");
    for ticker in tickers {
        let mut account = Account::new(BacktestBroker { capital: 1000.0 });
        let mut price_data = PriceData::new(polygon::client(&env));

        if let Some(candles) = price_data.history(ticker, 15, "1:minute") {
            let mut strategy = strategies::SmaCrossover::new(ticker, candles);
            strategy.execute(&mut price_data, &mut account);
            log_results(ticker, account);
        } else {
            break;
        }
    }
}

fn log_results(ticker: &String, account: Account<BacktestBroker>) {
    let mut winning_trades = Vec::new();
    let mut losing_trades = Vec::new();
    for position in &account.positions {
        if position.total_return() >= 0.0 {
            winning_trades.push(position);
        } else {
            losing_trades.push(position);
        }
    }
    winning_trades.sort_by(|a, b| b.total_return().partial_cmp(&a.total_return()).unwrap());
    losing_trades.sort_by(|a, b| a.total_return().partial_cmp(&b.total_return()).unwrap());
    let wins_sum: f64 = winning_trades.iter().map(|p| p.total_return()).sum();
    let losses_sum: f64 = losing_trades.iter().map(|p| p.total_return()).sum();

    let win_percent = winning_trades.len() as f64 / account.positions.len() as f64 * 100.0;
    println!(
        "{:6}-- W/L/W%: {}/{}/{:.2}% - P/L: ${:.4}/${:.4} - Net: ${:.4}",
        ticker,
        winning_trades.len(),
        losing_trades.len(),
        win_percent,
        wins_sum,
        losses_sum,
        wins_sum + losses_sum,
    );
}

#[cfg(test)]
mod tests {}
