use super::{
    apis::polygon,
    clock, config, strategies,
    trading::{Account, PriceData, SimBroker},
};

pub fn run_simulation(tickers: &[String], env: &config::Env) {
    println!("Only running simulation for first ticker {:?}", tickers[0]);
    let broker = SimBroker::new();
    let mut account: Account<SimBroker> = Account::new(broker);
    let mut price_data = PriceData::new(polygon::client(&env));

    if let Some(candles) = price_data.history(&tickers[0], 9, "1:minute") {
        let mut strategy = strategies::SmaCrossover::new(&tickers[0], candles);
        strategy.execute(&mut price_data, &mut account);
        log_results(account);
    }
}

fn log_results(mut account: Account<SimBroker>) {
    let mut winning_trades = Vec::new();
    let mut losing_trades = Vec::new();
    for position in &account.positions {
        if position.total_return() >= 0.0 {
            winning_trades.push(position);
        } else {
            losing_trades.push(position);
        }
    }
    winning_trades.sort_by(|a, b| b.total_return().partial_cmp(&a.total_return()).unwrap());
    losing_trades.sort_by(|a, b| a.total_return().partial_cmp(&b.total_return()).unwrap());
    let wins_sum: f64 = winning_trades.iter().map(|p| p.total_return()).sum();
    let losses_sum: f64 = losing_trades.iter().map(|p| p.total_return()).sum();

    let win_percent = winning_trades.len() as f64 / account.positions.len() as f64 * 100.0;
    println!(
        "W/L/W%: {}/{}/{:.2}% - P/L: ${:.4}/${:.4} - Net: ${:.4}",
        winning_trades.len(),
        losing_trades.len(),
        win_percent,
        wins_sum,
        losses_sum,
        wins_sum + losses_sum,
    );

    let time = clock::milliseconds_to_date(0);
    println!("Ending Capital: ${:.4}", account.final_capital(time));
}
