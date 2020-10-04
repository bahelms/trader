use super::{
    clock, studies,
    trading::{Account, PriceData},
};
use crate::apis::candles::Candle;

// Buy when price closes above SMA9.
// Sell when price closes below SMA9.
pub fn sma_crossover(ticker: &String, price_data: &mut PriceData, account: &mut Account) {
    let mut setup = false;
    let start_date = clock::days_ago(60);
    let candles = price_data.history(ticker, start_date, 9, "1:minute");
    let mut last_candle = &candles[0];

    // init studies
    let mut sma9 = studies::SMA::new(9);
    for candle in candles {
        sma9.add(candle.close);
        last_candle = candle;
    }

    if last_candle.close < sma9.value.unwrap() {
        setup = true;
    }

    while let Some(candle) = price_data.next_candle() {
        sma9.add(candle.close);
        let sma9_value = sma9.value.unwrap();

        if entry_signal(candle, sma9_value, setup) {
            let shares = account.max_shares(candle.close, candle.datetime);
            account.open_position(ticker, candle.close, shares, candle.datetime);
            setup = false;
        } else if exit_signal(candle, sma9_value, account) {
            account.close_position(ticker, candle.close, candle.datetime);
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
