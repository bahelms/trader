use std::io::Write;
use std::{fs, fs::File};

use super::candles::Candle;
use crate::config;
use chrono::{DateTime, FixedOffset, NaiveDateTime};

pub struct Client<'a> {
    client_id: &'a String,
    access_token: String,
    refresh_token: String,
    base_url: &'static str,
}

pub fn client(env: &config::Env) -> Client {
    let (access_token, refresh_token) = read_tokens();

    Client {
        access_token,
        refresh_token,
        client_id: &env["TD_CLIENT_ID"],
        base_url: "https://api.tdameritrade.com/v1",
    }
}

fn read_tokens() -> (String, String) {
    let access_token = fs::read_to_string("tokens/.td_access_token")
        .expect("couldn't open file: .td_access_token")
        .trim()
        .to_string();
    let refresh_token = fs::read_to_string("tokens/.td_refresh_token")
        .expect("couldn't open file: .td_refresh_token")
        .trim()
        .to_string();
    (access_token, refresh_token)
}

impl<'a> Client<'a> {
    pub fn price_history(&mut self, symbol: &str) -> Vec<Candle> {
        let url = format!("{}/marketdata/{}/pricehistory", self.base_url, symbol);
        let auth_header = format!("Bearer {}", self.access_token);
        let mut res = ureq::get(&url)
            .set("Authorization", &auth_header)
            .query("apiKey", &self.client_id)
            .query("periodType", "day")
            .query("period", "1")
            .query("frequencyType", "minute")
            .query("frequency", "1")
            .call();
        if res.status() == 401 {
            self.refresh_token();
            let auth_header = format!("Bearer {}", self.access_token);
            res = ureq::get(&url)
                .set("Authorization", &auth_header)
                .query("apiKey", &self.client_id)
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
    fn refresh_token(&mut self) {
        println!("Refreshing access token");
        let url = format!("{}/oauth2/token", self.base_url);
        let data = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &self.refresh_token),
            ("client_id", &self.client_id),
        ];
        let res = ureq::post(&url).send_form(&data);
        let json = res.into_json().unwrap();
        self.access_token = json["access_token"].as_str().unwrap().to_string();

        match File::create("tokens/.td_access_token") {
            Ok(mut file) => {
                file.write(json["access_token"].as_str().unwrap().as_bytes())
                    .expect("Error writing access token to file");
            }
            Err(err) => eprintln!("Error creating .td_access_token file: {}", err),
        }
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
