use super::{
    apis::alpha_vantage,
    clock, config, strategies,
    trading::{Account, Broker, PriceData},
};

const COMMISSION: f64 = 0.01; // TD Ameritrade's trade commission

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

pub fn run_simulation(tickers: &[String], env: &config::Env) {
    println!("Running simulation for {}", tickers[0]);
    let broker = SimBroker::new();
    let mut account: Account<SimBroker> = Account::new(broker);
    let mut price_data = PriceData::new(alpha_vantage::client(&env));

    if let Some(candles) = price_data.history(&tickers[0], 9, "1:minute") {
        let mut strategy = strategies::SmaCrossover::new(&tickers[0], candles);
        strategy.execute(&mut price_data, &mut account);
        log_results(account);
    }
}

fn log_results(mut account: Account<SimBroker>) {
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

    for position in &account.positions {
        println!("{}", position);
    }
    println!(
        "W/L/W%: {}/{}/{:.2}% - P/L: ${:.4}/${:.4} - Net: ${:.4}",
        winning_trades.len(),
        losing_trades.len(),
        win_percent,
        wins_sum,
        losses_sum,
        wins_sum + losses_sum,
    );

    let time = clock::milliseconds_to_date(0);
    println!("Ending Capital: ${:.4}", account.total_cash(time));
}

#[cfg(test)]
mod tests {
    use super::SimBroker;
    use crate::{
        apis::candles::Candle, clock, trading::Account, trading::Broker, trading::Position,
    };

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
