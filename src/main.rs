mod apis;
mod config;

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

    for candle in tda_client.price_history(&symbol) {
        println!("{}: {}", symbol, candle);
    }
}
