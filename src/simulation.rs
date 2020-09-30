use super::{apis::polygon, clock, config, strategies, trading::Account};

pub fn run_simulation(symbols: &[String], env: &config::Env) {
    println!("Only running simulation for first symbol {:?}", symbols[0]);
    const MINUTES: i32 = 1;
    let start_date = clock::weeks_ago(1);
    let end_date = clock::current_date();

    let mut account = Account::new(1000.0);
    let data_source = polygon::client(&env);

    let candles = data_source.price_history(&symbols[0], start_date, end_date, MINUTES);
    strategies::sma_crossover(&mut account, &candles);
    account.log_results();
}
