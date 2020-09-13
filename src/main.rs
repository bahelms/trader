use chrono::{DateTime, FixedOffset, NaiveDateTime};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::{fmt, fs, fs::File};

type Config = HashMap<String, String>;

struct Candle {
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: i64,
    time: String,
}

impl Candle {
    fn new(open: f64, close: f64, high: f64, low: f64, volume: i64, time: String) -> Self {
        Self {
            open,
            close,
            high,
            low,
            volume,
            time,
        }
    }
}

impl fmt::Display for Candle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, O: {}, C: {}, H: {}, L: {}, V: {}",
            self.time, self.open, self.close, self.high, self.low, self.volume
        )
    }
}

fn init_config() -> Config {
    let mut config = HashMap::new();
    add_env_file(&mut config);
    add_access_key(&mut config);
    config
}

fn add_env_file(config: &mut Config) {
    let file = match File::open(".env") {
        Ok(file) => file,
        Err(_) => panic!("couldn't open file: .env"),
    };

    for line in BufReader::new(file).lines() {
        let key_values: Vec<String> = line.unwrap().split("::").map(str::to_string).collect();
        config.insert(key_values[0].clone(), key_values[1].clone());
    }
}

fn add_access_key(config: &mut Config) {
    let token = fs::read_to_string(".td_access_token").expect("couldn't open file: .env");
    config.insert("TD_ACCESS_TOKEN".to_string(), token);
}

// TODO: This will need to handle refreshing the refresh token after 90 days, also.
// "expires_in": seconds
fn refresh_token(config: &mut Config) {
    let base_url = "https://api.tdameritrade.com/v1";
    let url = format!("{}/oauth2/token", base_url);
    let data = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &config["TD_REFRESH_TOKEN"]),
        ("client_id", &config["TD_CLIENT_ID"]),
    ];
    let res = ureq::post(&url).send_form(&data);
    let json = res.into_json().unwrap();

    config.insert(
        "TD_ACCESS_TOKEN".to_string(),
        json["access_token"].to_string(),
    );
    match File::create(".td_access_token") {
        Ok(mut file) => {
            file.write(json["access_token"].as_str().unwrap().as_bytes())
                .expect("Error writing access token to file");
        }
        Err(err) => eprintln!("Error creating .td_access_token file: {}", err),
    }
}

fn get_candles(ticker: &str, config: &mut Config) -> Vec<Candle> {
    let base_url = "https://api.tdameritrade.com/v1";
    let url = format!("{}/marketdata/{}/pricehistory", base_url, ticker);
    let auth_header = format!("Bearer {}", config["TD_ACCESS_TOKEN"]);
    let mut res = ureq::get(&url)
        .set("Authorization", &auth_header)
        .query("apiKey", &config["TD_CLIENT_ID"])
        .query("periodType", "day")
        .query("period", "1")
        .query("frequencyType", "minute")
        .query("frequency", "1")
        .call();
    if res.status() == 401 {
        refresh_token(config);
        let auth_header = format!("Bearer {}", config["TD_ACCESS_TOKEN"]);
        res = ureq::get(&url)
            .set("Authorization", &auth_header)
            .query("apiKey", &config["TD_CLIENT_ID"])
            .query("periodType", "day")
            .query("period", "1")
            .query("frequencyType", "minute")
            .query("frequency", "1")
            .call();
    }
    let json = res.into_json().unwrap();
    json["candles"]
        .as_array()
        .expect("candles JSON error")
        .iter()
        .map(|candle| {
            let date = milliseconds_to_date(candle["datetime"].as_i64().unwrap());
            Candle::new(
                candle["open"].as_f64().unwrap(),
                candle["close"].as_f64().unwrap(),
                candle["high"].as_f64().unwrap(),
                candle["low"].as_f64().unwrap(),
                candle["volume"].as_i64().unwrap(),
                date,
            )
        })
        .collect()
}

fn milliseconds_to_date(ms: i64) -> String {
    const HOURS: i32 = 3600;
    let seconds = ms / 1000;
    let naive_date = NaiveDateTime::from_timestamp(seconds, 0);
    let est = FixedOffset::west(4 * HOURS);
    DateTime::<FixedOffset>::from_utc(naive_date, est)
        .format("%D %l:%M:%S %p %z")
        .to_string()
}

fn main() {
    let mut config = init_config();

    for candle in get_candles("ETON", &mut config) {
        println!("ETON: {}", candle);
    }
}
