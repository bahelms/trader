pub mod candles;
pub mod td_ameritrade;

use ureq::Response;

fn get(url: &str, auth_header: String, params: &Vec<(&str, &str)>) -> Response {
    let mut request = ureq::get(url).set("Authorization", &auth_header).build();
    for (key, value) in params {
        request.query(key, value);
    }
    request.call()
}
