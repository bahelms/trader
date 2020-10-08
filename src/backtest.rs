use super::{
    apis::polygon,
    clock, config, strategies,
    trading::{Account, Broker, PriceData},
};

pub struct NullBroker {
    capital: f64,
}

impl Broker for NullBroker {
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
    let mut account = Account::new(NullBroker { capital: 1000.0 });

    for ticker in tickers {
        let mut price_data = PriceData::new(polygon::client(&env));
        let candles = price_data.history(ticker, 9, "1:minute");
        let mut strategy = strategies::SmaCrossover::new(ticker, candles);
        strategy.execute(&mut price_data, &mut account);
    }
    account.log_results();
}

#[cfg(test)]
mod tests {}
