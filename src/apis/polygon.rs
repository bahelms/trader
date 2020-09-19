use super::candles::Candle;
use crate::{clock, config, trading::PricePeriod};

pub fn client(env: &config::Env) -> Client {
    Client {
        api_key: &env["POLYGON_API_KEY"],
        base_url: "https://api.polygon.io/v2",
    }
}

pub struct Client<'a> {
    api_key: &'a String,
    base_url: &'static str,
}

impl<'a> Client<'a> {
    pub fn price_history(&self, symbol: &str, price_period: &PricePeriod) -> Vec<Candle> {
        let url = format!(
            "{}/aggs/ticker/{}/range/{}/{}/{}/{}",
            self.base_url,
            symbol,
            price_period.frequency,
            price_period.frequency_type,
            price_period.start_date,
            price_period.end_date
        );
        let params = vec![("apiKey", self.api_key)];
        let res = super::get(&url, String::new(), &params);
        let json = res.into_json().unwrap();
        json["results"]
            .as_array()
            .expect("candles JSON error")
            .iter()
            .map(format_candle)
            .collect()
    }
}

fn format_candle(candle: &serde_json::value::Value) -> Candle {
    let date = clock::milliseconds_to_date(candle["t"].as_i64().unwrap());
    // handle volume coming in as a float
    let volume = match candle["v"].as_i64() {
        Some(v) => v,
        None => candle["v"].as_f64().unwrap() as i64,
    };
    Candle::new(
        candle["o"].as_f64().unwrap(),
        candle["c"].as_f64().unwrap(),
        candle["h"].as_f64().unwrap(),
        candle["l"].as_f64().unwrap(),
        volume,
        date,
    )
}