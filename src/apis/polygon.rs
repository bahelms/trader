use super::candles::Candle;
use crate::{clock, config};
use serde::Deserialize;
use serde_json::{json, value::Value};
use std::{
    fs,
    io::{BufReader, Write},
    path::PathBuf,
    time::SystemTime,
};

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
    pub fn price_history(
        &self,
        ticker: &str,
        start_date: clock::DateWithoutTZ,
        end_date: clock::DateWithoutTZ,
        frequency: String,
        frequency_type: String,
    ) -> Option<Vec<Candle>> {
        if let Ok(entries) = fs::read_dir(cache_path()) {
            for entry in entries {
                let entry_path = entry.unwrap().path();
                let timestamp = entry_path.metadata().unwrap().created().unwrap();
                let dur = timestamp
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                let file_date = clock::milliseconds_to_date(dur * 1000).date().naive_local();

                if entry_path.file_stem().unwrap() == ticker && file_date == clock::current_date() {
                    let file = fs::File::open(entry_path).expect("Error opening file to write!");
                    let reader = BufReader::new(file);
                    let holder: CandleHolder =
                        serde_json::from_reader(reader).expect("Error reading JSON file!");
                    return Some(holder.candles.iter().map(format_candle).collect());
                }
            }
        }

        println!("requesting {} from API", ticker);
        let url = format!(
            "{}/aggs/ticker/{}/range/{}/{}/{}/{}",
            self.base_url, ticker, frequency, frequency_type, start_date, end_date
        );
        let params = vec![("apiKey", self.api_key)];
        let res = super::get(&url, String::new(), &params);
        let response_status = res.status_text().to_string();
        let json = res.into_json().unwrap();

        if let Some(results) = json["results"].as_array() {
            cache_results(ticker, results);
            Some(results.iter().map(format_candle).collect())
        } else {
            eprintln!(
                "\nPolygon.price_history response error: {:?}",
                response_status
            );
            None
        }
    }
}

#[derive(Deserialize)]
// exists solely to use serde deserialization from reader
struct CandleHolder {
    candles: Vec<Value>,
}

fn cache_path() -> PathBuf {
    let mut cache_path = PathBuf::new();
    cache_path.push("backtest_cache");
    cache_path
}

fn cache_results(ticker: &str, results: &Vec<serde_json::value::Value>) {
    let mut path = cache_path();
    if !path.is_dir() {
        fs::create_dir(&path).expect("Error creating cache directory!");
    }

    path.push(ticker);
    path.set_extension("json");
    let mut file = fs::File::create(path).expect("Error creating cache file!");

    let json = json!({ "candles": results });
    file.write(serde_json::to_string(&json).unwrap().as_bytes())
        .expect("Error writing candle to file!");
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
