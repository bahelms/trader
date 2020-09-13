use std::fs::File;
use std::io::Write;

use super::candles::Candle;
use crate::config::Config;
use chrono::{DateTime, FixedOffset, NaiveDateTime};

pub fn get_candles(ticker: &str, config: &mut Config) -> Vec<Candle> {
    let base_url = "https://api.tdameritrade.com/v1";
    let url = format!("{}/marketdata/{}/pricehistory", base_url, ticker);
    let auth_header = format!("Bearer {}", config["TD_ACCESS_TOKEN"]);
    println!("auth: {}", auth_header);
    let mut res = ureq::get(&url)
        .set("Authorization", &auth_header)
        .query("apiKey", &config["TD_CLIENT_ID"])
        .query("periodType", "day")
        .query("period", "1")
        .query("frequencyType", "minute")
        .query("frequency", "1")
        .call();
    if res.status() == 401 {
        println!("old auth: {}", auth_header);
        refresh_token(config);
        let auth_header = format!("Bearer {}", config["TD_ACCESS_TOKEN"]);
        println!("new auth: {}", auth_header);
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

// TODO: This will need to handle refreshing the refresh token after 90 days, also.
// "expires_in": seconds
fn refresh_token(config: &mut Config) {
    println!("Refreshing access token");
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
        json["access_token"].as_str().unwrap().to_string(),
    );
    match File::create(".td_access_token") {
        Ok(mut file) => {
            file.write(json["access_token"].as_str().unwrap().as_bytes())
                .expect("Error writing access token to file");
        }
        Err(err) => eprintln!("Error creating .td_access_token file: {}", err),
    }
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
