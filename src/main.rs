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
    let mut trades = Backtest::new(1000.00);

    let candles = tda_client.price_history(&symbol);
    if candles.len() < 9 {
        eprintln!("Not enough candles for minimum trading: {}", candles.len());
        return;
    }

    trades = strategies::sma9_crossover(&candles, trades);
    log_results(trades);
}

fn log_results(trades: Backtest) {
    let mut winning_trades = Vec::new();
    let mut losing_trades = Vec::new();
    for position in &trades.positions {
        if position.total_return() >= 0.0 {
            winning_trades.push(position);
        } else {
            losing_trades.push(position);
        }
    }
    let mut winning_returns: Vec<f64> = winning_trades.iter().map(|p| p.total_return()).collect();
    let mut losing_returns: Vec<f64> = losing_trades.iter().map(|p| p.total_return()).collect();
    let total_wins: f64 = winning_returns.iter().sum();
    let total_losses: f64 = losing_returns.iter().sum();
    winning_returns.sort_by(|a, b| b.partial_cmp(a).unwrap());
    losing_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!(
        "Trades won: {} - total returns: ${}",
        winning_trades.len(),
        total_wins
    );
    println!(
        "Trades lost: {} - total returns: ${}",
        losing_trades.len(),
        total_losses
    );
    println!(
        "Win %: {:.2}",
        winning_trades.len() as f64 / trades.positions.len() as f64 * 100.0
    );
    println!(
        "Highest return: ${} - Lowest return: ${}",
        winning_returns[0], losing_returns[0],
    );

    println!("Net return: ${}", total_wins + total_losses);
    println!("Ending Capital: ${}", trades.capital);
}
