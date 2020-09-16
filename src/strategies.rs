use super::{clock, studies, trading::Trades};
use crate::apis::candles::Candle;

// Buy when price closes above SMA9.
// Sell when price closes below SMA9.
pub fn confirmation_above_sma<T: Trades>(candles: &Vec<Candle>, mut trades: T) -> T {
    let bar9 = 9;
    let mut setup = false;
    let mut sma9 = studies::sma(&closed_prices(&candles[..bar9]), bar9);

    if candles[bar9 - 1].close < sma9 {
        setup = true;
    }

    for (i, candle) in candles[bar9..].iter().enumerate() {
        let end = i + bar9;
        let start = end - (bar9 - 1);
        sma9 = studies::sma(&closed_prices(&candles[start..end]), bar9);

        if candle.close > sma9 && setup {
            if outside_market_hours(candle.time()) {
                continue;
            }

            let shares = trades.max_purchaseable_shares(candle.close);
            trades.open_position(candle.close, shares);
            setup = false;
        } else if candle.close < sma9 && trades.is_current_position_open() {
            if outside_market_hours(candle.time()) {
                continue;
            }

            trades.close_current_position(candle.close);
        } else if candle.close < sma9 && !trades.is_current_position_open() {
            setup = true;
        }
    }
    trades
}

fn closed_prices(candles: &[Candle]) -> Vec<f64> {
    candles.iter().map(|candle| candle.close).collect()
}

fn outside_market_hours(time: clock::Time) -> bool {
    let (open, close) = clock::market_hours();
    time < open || time > close
}
