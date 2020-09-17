use super::candles::Candle;
use crate::{clock, config};
use std::{fs, fs::File, io::Write};

pub fn client(env: &config::Env) -> Client {
    Client {
        access_token: read_token_file("access"),
        refresh_token: read_token_file("refresh"),
        client_id: &env["TD_CLIENT_ID"],
        base_url: "https://api.tdameritrade.com/v1",
    }
}

pub struct Client<'a> {
    client_id: &'a String,
    access_token: String,
    refresh_token: String,
    base_url: &'static str,
}

impl<'a> Client<'a> {
    pub fn price_history(
        &mut self,
        symbol: &str,
        period_type: &str,
        period: &str,
        fq_type: &str,
        fq: &str,
    ) -> Vec<Candle> {
        let url = format!("{}/marketdata/{}/pricehistory", self.base_url, symbol);
        let params = vec![
            ("periodType", period_type),
            ("period", period),
            ("frequencyType", fq_type),
            ("frequency", fq),
            ("apiKey", self.client_id),
        ];

        let mut res = super::get(&url, self.bearer_token(), &params);
        if res.status() == 401 {
            self.refresh_token();
            res = super::get(&url, self.bearer_token(), &params);
        }

        let json = res.into_json().unwrap();
        json["candles"]
            .as_array()
            .expect("candles JSON error")
            .iter()
            .map(format_candle)
            .collect()
    }

    fn bearer_token(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    // TODO: This will need to handle refreshing the refresh token after 90 days, also.
    // "expires_in": seconds
    fn refresh_token(&mut self) {
        let url = format!("{}/oauth2/token", self.base_url);
        let data = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &self.refresh_token),
            ("client_id", &self.client_id),
        ];
        let res = ureq::post(&url).send_form(&data);
        let json = res.into_json().unwrap();
        let token_str = json["access_token"].as_str().unwrap();

        self.access_token = token_str.to_string();
        match File::create("tokens/.td_access_token") {
            Ok(mut file) => {
                file.write(token_str.as_bytes())
                    .expect("Error writing access token to file");
            }
            Err(err) => eprintln!("Error creating .td_access_token file: {}", err),
        }
    }
}

fn read_token_file(token_type: &str) -> String {
    fs::read_to_string(format!("tokens/.td_{}_token", token_type))
        .expect(&format!("couldn't open file: .td_{}_token", token_type))
        .trim()
        .to_string()
}

fn format_candle(candle: &serde_json::value::Value) -> Candle {
    let date = clock::milliseconds_to_date(candle["datetime"].as_i64().unwrap());
    Candle::new(
        candle["open"].as_f64().unwrap(),
        candle["close"].as_f64().unwrap(),
        candle["high"].as_f64().unwrap(),
        candle["low"].as_f64().unwrap(),
        candle["volume"].as_i64().unwrap(),
        date,
    )
}
