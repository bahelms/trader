pub fn sma(prices: &Vec<f64>, bars: usize) -> f64 {
    let bars = if bars > prices.len() {
        prices.len()
    } else {
        bars
    };

    let start = prices.len() - bars;
    let sum: f64 = prices[start..].iter().sum();
    sum / bars as f64
}

#[cfg(test)]
mod tests {
    use super::sma;

    const PRICES: [f64; 7] = [12.5432, 13.3, 12.9, 17.6, 16.34, 18.78, 16.39018];

    #[test]
    fn sma_averages_starting_from_the_end_of_the_vector() {
        let prices = PRICES.to_vec();
        assert_eq!(sma(&prices, 3), 17.170060000000003);
        assert_eq!(sma(&prices, 7), 15.407625714285714);
    }

    #[test]
    fn sma_uses_all_prices_when_bars_are_greater_than_length() {
        let prices = PRICES.to_vec();
        assert_eq!(sma(&prices, 180), 15.407625714285714);
    }
}
