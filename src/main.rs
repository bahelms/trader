mod apis;
// mod backtest;
mod clock;
mod config;
mod simulation;
mod strategies;
mod studies;
mod trading;

// use backtest::backtest;
use std::env;

fn main() {
    let args: Vec<String> = env::args().map(|a| a.to_uppercase()).collect();
    if args.len() < 2 {
        eprintln!("Must provide at least one symbol to use");
        return;
    }

    let env = config::init_env();
    match args[1].as_str() {
        "--BACKTEST" => {
            println!("Backtesting is broken currently")
            // let symbol = args[1].to_uppercase();
            // backtest(symbol, apis::polygon::client(&env), 1000.0);
        }
        "--SIM" => {
            simulation::run_simulation(&args[2..], &env);
        }
        "--PAPER" => println!("Paper trading not implemented yet"),
        _ => println!("Live trading not implemented yet"),
    }
}
