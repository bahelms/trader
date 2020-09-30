use super::{
    apis::polygon,
    clock, config, strategies,
    trading::{Account, Broker},
};

pub fn run_simulation(tickers: &[String], env: &config::Env) {
    println!("Only running simulation for first ticker {:?}", tickers[0]);
    const MINUTES: i32 = 1;
    let start_date = clock::days_ago(1);
    let end_date = clock::current_date();

    let mut account = Account::new(Broker {});
    let data_source = polygon::client(&env);

    let candles = data_source.price_history(&tickers[0], start_date, end_date, MINUTES);
    strategies::sma_crossover(&mut account, &candles, &tickers[0]);
    account.log_results();
}
