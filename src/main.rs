mod apis;
mod config;
mod studies;

use std::env;

use apis::td_ameritrade;

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
    let closed_prices = candles.iter().map(|candle| candle.close).collect();
    let sma9 = studies::sma(&closed_prices, 9);

    for candle in candles {
        println!("{}: {}", symbol, candle);
    }
    println!("SMA9 {}", sma9);
}
