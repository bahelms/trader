mod apis;
mod config;

use apis::td_ameritrade;

fn main() {
    let symbol = "AAPL";
    let mut config = config::init_config();

    for candle in td_ameritrade::get_candles(symbol, &mut config) {
        println!("{}: {}", symbol, candle);
    }
}
