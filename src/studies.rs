pub struct SMA {
    pub value: Option<f64>,
    bars: usize,
    buffer: Vec<f64>,
}

impl SMA {
    pub fn new(bars: usize) -> Self {
        Self {
            bars,
            value: None,
            buffer: Vec::new(),
        }
    }

    pub fn add(&mut self, price: f64) {
        self.buffer.push(price);
        if self.buffer.len() < self.bars {
            return;
        }

        if self.buffer.len() > self.bars {
            self.buffer.remove(0);
        }
        self.value = Some(self.buffer.iter().sum::<f64>() / self.bars as f64);
    }
}

#[cfg(test)]
mod tests {
    use super::SMA;

    #[test]
    fn sma_adding_prices_below_bar_length_does_not_calculate_value() {
        let mut sma = SMA::new(3);
        sma.add(23.1);
        assert_eq!(sma.value, None);
        sma.add(10.45);
        assert_eq!(sma.value, None);
    }

    #[test]
    fn sma_value_is_calculated_when_the_number_of_prices_matches_the_bars_length() {
        let mut sma = SMA::new(2);
        sma.add(23.1);
        sma.add(10.45);
        assert_eq!(sma.value.unwrap(), 16.775);
    }

    #[test]
    fn sma_the_average_moves_based_on_bar_length() {
        let mut sma = SMA::new(2);
        sma.add(23.1);
        sma.add(10.45);
        sma.add(4.32);
        assert_eq!(sma.value.unwrap(), 7.385);
        sma.add(54.2);
        assert_eq!(sma.value.unwrap(), 29.26);
    }

    //     #[test]
    //     fn sma_averages_starting_from_the_end_of_the_vector() {
    //         let prices = PRICES.to_vec();
    //         assert_eq!(sma(&prices, 3), 17.170060000000003);
    //         assert_eq!(sma(&prices, 7), 15.407625714285714);
    //     }

    //     #[test]
    //     fn sma_uses_all_prices_when_bars_are_greater_than_length() {
    //         let prices = PRICES.to_vec();
    //         assert_eq!(sma(&prices, 180), 15.407625714285714);
    //     }
}
