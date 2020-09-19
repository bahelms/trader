use chrono::{DateTime, FixedOffset, NaiveDateTime, NaiveTime};
use std::time::{SystemTime, UNIX_EPOCH};

pub type DateEST = DateTime<FixedOffset>;
pub type Time = NaiveTime;

const HOURS: i32 = 3600;

pub fn market_hours() -> (NaiveTime, NaiveTime) {
    (NaiveTime::from_hms(9, 30, 0), NaiveTime::from_hms(16, 0, 0))
}

pub fn outside_market_hours(time: Time) -> bool {
    let (open, close) = market_hours();
    time < open || time >= close
}

pub fn current_datetime() -> DateEST {
    milliseconds_to_date(current_milliseconds() as i64)
}

pub fn milliseconds_to_date(ms: i64) -> DateEST {
    let seconds = ms / 1000;
    let naive_date = NaiveDateTime::from_timestamp(seconds, 0);
    let est = FixedOffset::west(4 * HOURS);
    DateTime::<FixedOffset>::from_utc(naive_date, est)
}

pub fn current_milliseconds() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting milliseconds from epoch")
        .as_millis()
}
