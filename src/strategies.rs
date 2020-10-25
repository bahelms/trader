use super::{
    studies,
    trading::{Account, Broker, PriceData},
};
use crate::apis::candles::Candle;

pub struct SmaCrossover<'a> {
    setup: bool,
    sma9: studies::SMA,
    ticker: &'a String,
}

pub struct Sma9CrossesSma180<'a> {
    setup: bool,
    sma9: studies::SMA,
    sma180: studies::SMA,
    ticker: &'a String,
}

// Buy when price closes above SMA9.
// Sell when price closes below SMA9.
impl<'a> SmaCrossover<'a> {
    pub fn new(ticker: &'a String, candles: &[Candle]) -> Self {
        // init studies
        let mut sma9 = studies::SMA::new(9);
        for candle in candles {
            sma9.add(candle.close);
        }

        Self {
            setup: candles.last().unwrap().close < sma9.value.unwrap(),
            sma9,
            ticker,
        }
    }

    pub fn entry_signal(&mut self, candle: &Candle) -> bool {
        let sma9_value = self.sma9.value.unwrap();
        candle.close > sma9_value && candle.is_bull() && self.setup
    }

    pub fn exit_signal(&mut self, candle: &Candle) -> bool {
        let sma9_value = self.sma9.value.unwrap();
        candle.close < sma9_value && candle.is_bear()
    }

    pub fn setup_found(&mut self, candle: &Candle) -> bool {
        let sma9_value = self.sma9.value.unwrap();
        candle.close < sma9_value
    }

    pub fn execute<B: Broker>(&mut self, price_data: &mut PriceData, account: &mut Account<'a, B>) {
        while let Some(candle) = price_data.next_candle() {
            self.sma9.add(candle.close);

            if self.entry_signal(candle) {
                let shares = account.max_shares(candle.close, candle.datetime);
                account.open_position(self.ticker, candle.close, shares, candle.datetime);
                self.setup = false;
            } else if self.exit_signal(candle) && account.is_position_open() {
                account.close_position(self.ticker, candle.close, candle.datetime);
            } else if self.setup_found(candle) && !account.is_position_open() {
                self.setup = true;
            }

            account.close_position_for_day(self.ticker, candle);
        }
    }
}

// Buy when SMA9 crosses above SMA180.
// Sell when price closes below SMA9.
impl<'a> Sma9CrossesSma180<'a> {
    pub fn new(ticker: &'a String, candles: &[Candle]) -> Self {
        // init studies
        let mut sma9 = studies::SMA::new(9);
        let mut sma180 = studies::SMA::new(180);
        for candle in candles {
            sma9.add(candle.close);
            sma180.add(candle.close);
        }

        Self {
            setup: sma9.value.unwrap() < sma180.value.unwrap(),
            sma9,
            sma180,
            ticker,
        }
    }

    pub fn entry_signal(&self) -> bool {
        let sma9_value = self.sma9.value.unwrap();
        let sma180_value = self.sma180.value.unwrap();
        sma9_value > sma180_value && self.setup
    }

    pub fn exit_signal(&self, candle: &Candle) -> bool {
        let sma9_value = self.sma9.value.unwrap();
        candle.close < sma9_value && candle.is_bear()
    }

    pub fn setup_found(&self) -> bool {
        self.sma9.value.unwrap() < self.sma180.value.unwrap()
    }

    pub fn execute<B: Broker>(&mut self, price_data: &mut PriceData, account: &mut Account<'a, B>) {
        while let Some(candle) = price_data.next_candle() {
            self.sma9.add(candle.close);
            self.sma180.add(candle.close);

            if self.entry_signal() {
                let shares = account.max_shares(candle.close, candle.datetime);
                account.open_position(self.ticker, candle.close, shares, candle.datetime);
                self.setup = false;
            } else if self.exit_signal(candle) && account.is_position_open() {
                account.close_position(self.ticker, candle.close, candle.datetime);
            } else if self.setup_found() && !account.is_position_open() {
                self.setup = true;
            }

            account.close_position_for_day(self.ticker, candle);
        }
    }
}
