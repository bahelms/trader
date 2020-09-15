mod apis;
mod clock;
mod config;
mod strategies;
mod studies;

use apis::td_ameritrade;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Must provide a symbol to use");
        return;
    }

    let symbol = args[1].to_uppercase();
    let env = config::init_env();
    let mut tda_client = td_ameritrade::client(&env);

    let candles = tda_client.price_history(&symbol);
    // for candle in &candles {
    //     println!("{}: {}", symbol, candle);
    // }
    strategies::confirmation_above_sma(&candles);
}
