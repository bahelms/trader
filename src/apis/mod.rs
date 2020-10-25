pub mod alpha_vantage;
pub mod candles;
pub mod polygon;
// pub mod td_ameritrade;

use std::path::PathBuf;
use ureq::Response;

fn get(url: &str, auth_header: String, params: &Vec<(&str, &String)>) -> Response {
    let mut request = ureq::get(url).set("Authorization", &auth_header).build();
    for (key, value) in params {
        request.query(key, value);
    }
    request.call()
}

fn cache_path() -> PathBuf {
    let mut cache_path = PathBuf::new();
    cache_path.push("backtest_cache");
    cache_path
}
