use super::{studies, trading::Account};
use crate::apis::candles::{Candle, Candles};

// Buy when price closes above SMA9.
// Sell when price closes below SMA9.
pub fn sma_crossover(account: &mut Account, candles: &Candles, ticker: &String) {
    let mut setup = false;
    let mut candles_iter = candles.iter();

    // init studies
    let mut sma9 = studies::SMA::new(9);
    for candle in &mut candles_iter {
        sma9.add(candle.close);
        if sma9.value.is_some() {
            break;
        }
    }

    if let Some(previous_candle) = candles_iter.previous_candle {
        if previous_candle.close < sma9.value.unwrap() {
            setup = true;
        }
    }

    for candle in &mut candles_iter {
        sma9.add(candle.close);
        let sma9_value = sma9.value.unwrap();

        if entry_signal(candle, sma9_value, setup) {
            let shares = account.max_shares(candle.close);
            account.open_position(ticker, candle.close, shares, candle.datetime);
            setup = false;
        } else if exit_signal(candle, sma9_value, account) {
            account.close_current_position(ticker, candle.close, candle.datetime);
        } else if candle.close < sma9_value && !account.is_current_position_open() {
            setup = true;
        }
    }
}

fn entry_signal(candle: &Candle, sma9_value: f64, setup: bool) -> bool {
    candle.close > sma9_value && candle.is_bull() && setup
}

fn exit_signal(candle: &Candle, sma9_value: f64, account: &Account) -> bool {
    candle.close < sma9_value && candle.is_bear() && account.is_current_position_open()
}

// fn close_day_position<T: Trades>(next_bar_time: clock::Time, trades: &T) -> bool {
//     clock::outside_market_hours(next_bar_time) && trades.is_current_position_open()
// }
