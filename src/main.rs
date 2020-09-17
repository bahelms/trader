mod apis;
mod clock;
mod config;
mod strategies;
mod studies;
mod trading;

use apis::td_ameritrade;
use std::env;
use trading::Backtest;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Must provide a symbol to use");
        return;
    }

    let symbol = args[1].to_uppercase();
    let env = config::init_env();
    let mut tda_client = td_ameritrade::client(&env);
    let mut trades = Backtest::new(1000.00, "10 Day - 1 minute chart");

    let candles = tda_client.price_history(&symbol, "day", "10", "minute", "1");
    if candles.len() < 9 {
        eprintln!("Not enough candles for minimum trading: {}", candles.len());
        return;
    }
    println!("First Candle {}", candles[0]);
    println!("Last Candle {}\n", candles.last().unwrap());

    trades = strategies::sma9_crossover(&candles, trades);
    trades.log_results();
}
