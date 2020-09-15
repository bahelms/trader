use super::studies;
use crate::apis::candles::Candle;

// Buy when price closes above SMA9.
// Sell when price closes below SMA9.
pub fn confirmation_above_sma(candles: &Vec<Candle>) {
    let bar9 = 9;
    let mut capital = 1000.00;
    let mut shares = 0;
    let mut cost = 0.0;
    let mut setup = false;
    let mut position_open = false;
    let mut sma9 = studies::sma(&closed_prices(&candles[..bar9]), bar9);
    if candles[bar9 - 1].close < sma9 {
        setup = true;
    }

    for (i, candle) in candles[bar9..].iter().enumerate() {
        let end = i + bar9;
        let start = end - (bar9 - 1);
        sma9 = studies::sma(&closed_prices(&candles[start..end]), bar9);
        if candle.close > sma9 && setup {
            // if candle.time between 9:30 - 4 EST
            shares = (capital / candle.close) as i32;
            cost = shares as f64 * candle.close;
            capital -= cost;
            println!("bought {} at ${} - {}", shares, candle.close, candle.time); // buy at market
            position_open = true;
            setup = false;
        } else if candle.close < sma9 && position_open {
            // if candle.time between 9:30 - 4 EST
            let ret = shares as f64 * candle.close;
            capital += ret;
            println!(
                "sold {} at ${} (${}) - {}",
                shares,
                candle.close,
                ret - cost,
                candle.time
            ); // sell at market
            position_open = false;
        } else if candle.close < sma9 && !position_open {
            setup = true;
        }
    }
    println!("capital: {}", capital);
}

fn closed_prices(candles: &[Candle]) -> Vec<f64> {
    candles.iter().map(|candle| candle.close).collect()
}
