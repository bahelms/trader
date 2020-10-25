use super::candles::Candle;
use crate::{clock, config};
use std::{fs, io::Write, time::SystemTime};

pub fn client(env: &config::Env) -> Client {
    Client {
        api_key: &env["ALPHA_VANTAGE_KEY"],
        base_url: "https://www.alphavantage.co/query",
    }
}

pub struct Client<'a> {
    api_key: &'a String,
    base_url: &'static str,
}

impl<'a> Client<'a> {
    pub fn price_history(
        &self,
        ticker: &String,
        _start_date: clock::DateWithoutTZ,
        _end_date: clock::DateWithoutTZ,
        _frequency: String,
        _frequency_type: String,
    ) -> Option<Vec<Candle>> {
        if let Ok(entries) = fs::read_dir(super::cache_path()) {
            for entry in entries {
                let entry_path = entry.unwrap().path();
                if entry_path.file_stem().unwrap() == ticker.as_str() {
                    let timestamp = entry_path.metadata().unwrap().modified().unwrap();
                    let dur = timestamp
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    let file_date = clock::milliseconds_to_date(dur * 1000).date().naive_local();

                    if file_date == clock::current_date() {
                        let csv =
                            fs::read_to_string(entry_path).expect("Error opening file to write!");
                        return csv_to_candles(csv);
                    }
                }
            }
        }

        println!("requesting {} from API", ticker);
        let function = "TIME_SERIES_INTRADAY_EXTENDED".to_string();
        let interval = "1min".to_string();
        let slice = "year1month1".to_string();
        let params = vec![
            ("apiKey", self.api_key),
            ("symbol", ticker),
            ("function", &function),
            ("interval", &interval),
            ("slice", &slice),
        ];
        let response = super::get(self.base_url, String::new(), &params);
        let response_status = response.status_text().to_string();

        if let Ok(csv) = response.into_string() {
            cache_results(ticker, &csv);
            csv_to_candles(csv)
        } else {
            eprintln!(
                "\nAlphaVantage.price_history response error: {:?}",
                response_status
            );
            None
        }
    }
}

fn cache_results(ticker: &str, results: &String) {
    let mut path = super::cache_path();
    if !path.is_dir() {
        fs::create_dir(&path).expect("Error creating cache directory!");
    }

    path.push(ticker);
    path.set_extension("csv");
    let mut file = fs::File::create(path).expect("Error creating cache file!");

    file.write(results.as_bytes())
        .expect("Error writing candle to file!");
}

fn csv_to_candles(csv: String) -> Option<Vec<Candle>> {
    let candles = csv
        .lines()
        .rev()
        .filter(|line| !line.starts_with("time"))
        .map(format_candle)
        .collect();
    Some(candles)
}

fn format_candle(candle: &str) -> Candle {
    // time,open,high,low,close,volume
    // 2020-09-21 04:36:00,1.28,1.28,1.28,1.28,100
    let values: Vec<&str> = candle.split(",").collect();
    let date = clock::parse_datetime(values[0]);

    Candle::new(
        values[1].parse::<f64>().unwrap(),
        values[2].parse::<f64>().unwrap(),
        values[3].parse::<f64>().unwrap(),
        values[4].parse::<f64>().unwrap(),
        values[5].parse::<i64>().unwrap(),
        date,
    )
}
