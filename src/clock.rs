use chrono::{DateTime, FixedOffset, NaiveDateTime, NaiveTime};
use std::time::{SystemTime, UNIX_EPOCH};

pub type DateEST = DateTime<FixedOffset>;
pub type Time = NaiveTime;

const HOURS: i32 = 3600;

pub fn market_hours() -> (NaiveTime, NaiveTime) {
    (NaiveTime::from_hms(9, 30, 0), NaiveTime::from_hms(16, 0, 0))
}

pub fn current_datetime() -> DateEST {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting milliseconds from epoch")
        .as_millis();
    milliseconds_to_date(ms as i64)
}

pub fn milliseconds_to_date(ms: i64) -> DateEST {
    let seconds = ms / 1000;
    let naive_date = NaiveDateTime::from_timestamp(seconds, 0);
    let est = FixedOffset::west(4 * HOURS);
    DateTime::<FixedOffset>::from_utc(naive_date, est)
}

pub fn current_milliseconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting milliseconds from epoch")
        .as_millis()
        .to_string()
}
