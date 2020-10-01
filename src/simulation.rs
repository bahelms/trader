use super::{
    apis::polygon,
    config, strategies,
    trading::{Account, Broker, PriceData},
};

pub fn run_simulation(tickers: &[String], env: &config::Env) {
    println!("Only running simulation for first ticker {:?}", tickers[0]);
    let broker = Broker {};
    let mut account = Account::new(broker);
    let mut price_data = PriceData::new(polygon::client(&env));

    strategies::sma_crossover(&tickers[0], &mut price_data, &mut account);
    account.log_results();
}
