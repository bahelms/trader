mod apis;
mod clock;
mod config;
mod strategies;
mod studies;
mod trading;

use apis::td_ameritrade;
use std::env;
use trading::{Backtest, Trades};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Must provide a symbol to use");
        return;
    }

    let symbol = args[1].to_uppercase();
    let env = config::init_env();
    let mut tda_client = td_ameritrade::client(&env);
    let mut trades = Backtest::new(1000.00);

    let candles = tda_client.price_history(&symbol);
    if candles.len() < 9 {
        eprintln!("Not enough candles for minimum trading: {}", candles.len());
        return;
    }

    trades = strategies::confirmation_above_sma(&candles, trades);
    log_results(trades);
}

fn log_results(trades: Backtest) {
    for position in &trades.positions {
        if !position.open {
            let close = &position.closes[0];
            let ret = close.ask * close.shares as f64 - position.bid * position.shares as f64;
            println!(
                "trade: ({}) {} shares at ${} - sold at ${} -- return ${:.4}",
                position.time, position.shares, position.bid, close.ask, ret
            );
        }
    }

    if trades.is_current_position_open() {
        let position = trades.current_position().unwrap();
        println!(
            "open position: {} shares, {} bid",
            position.shares, position.bid
        );
    }

    println!("capital: ${}", trades.capital);
}
