mod apis;
mod backtest;
mod clock;
mod config;
mod strategies;
mod studies;
mod trading;

use apis::td_ameritrade;
use backtest::backtest;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Must provide a symbol to use");
        return;
    }

    let symbol = args[1].to_uppercase();
    let env = config::init_env();
    backtest(symbol, td_ameritrade::client(&env), 1000.0);
}
