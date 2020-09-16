use super::clock;

pub struct Trades {
    pub positions: Vec<Position>,
    pub capital: f64,
    pub pdt_remaining: i32,
}

pub struct Position {
    pub open: bool,
    pub shares: i32,
    pub bid: f64,
    pub closes: Vec<Close>,
    pub time: clock::DateEST,
}

pub struct Close {
    pub shares: i32,
    pub ask: f64,
    pub time: clock::DateEST,
}

impl Trades {
    pub fn new(capital: f64) -> Self {
        Self {
            capital,
            positions: Vec::new(),
            pdt_remaining: 3,
        }
    }

    pub fn max_purchaseable_shares(&self, price: f64) -> i32 {
        (self.capital / price) as i32
    }

    pub fn current_position(&self) -> Option<&Position> {
        self.positions.last()
    }

    pub fn open_position(&mut self, bid: f64, shares: i32) {
        // send buy order
        self.capital -= bid * shares as f64;
        self.positions.push(Position::open(shares, bid));
    }

    pub fn is_current_position_open(&self) -> bool {
        if let Some(position) = self.current_position() {
            position.open
        } else {
            false
        }
    }

    pub fn close_current_position(&mut self, ask: f64) {
        // send sell order
        let mut position = self.positions.pop().unwrap();
        position.close(ask);
        self.capital += ask * position.shares as f64;
        self.positions.push(position);
    }
}

impl Position {
    pub fn open(shares: i32, bid: f64) -> Self {
        Self {
            open: true,
            shares,
            bid,
            closes: Vec::new(),
            time: clock::current_datetime(),
        }
    }

    pub fn close(&mut self, ask: f64) {
        self.open = false;
        self.closes = vec![Close {
            ask,
            shares: self.shares,
            time: clock::current_datetime(),
        }];
    }
}
