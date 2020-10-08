use super::{
    apis::polygon,
    config, strategies,
    trading::{Account, PriceData, SimBroker},
};

pub fn run_simulation(tickers: &[String], env: &config::Env) {
    println!("Only running simulation for first ticker {:?}", tickers[0]);
    let broker = SimBroker::new();
    let mut account: Account<SimBroker> = Account::new(broker);
    let mut price_data = PriceData::new(polygon::client(&env));

    let candles = price_data.history(&tickers[0], 9, "1:minute");
    let mut strategy = strategies::SmaCrossover::new(&tickers[0], candles);
    strategy.execute(&mut price_data, &mut account);
    account.log_results();
}
