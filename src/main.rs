mod apis;
mod config;

use apis::td_ameritrade;

fn main() {
    let env = config::init_env();
    let mut tda_client = td_ameritrade::client(&env);

    let symbol = "SQQQ";
    for candle in tda_client.price_history(symbol) {
        println!("{}: {}", symbol, candle);
    }
}
