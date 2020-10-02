pub use chrono::Duration;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime};

pub type DateEST = DateTime<FixedOffset>;
pub type Time = NaiveTime;
pub type Date = NaiveDate;

const HOURS: i32 = 3600;

pub fn current_date() -> Date {
    Local::now().date().naive_local()
}

pub fn market_hours() -> (Time, Time) {
    (Time::from_hms(9, 30, 0), Time::from_hms(16, 0, 0))
}

pub fn outside_market_hours(time: Time) -> bool {
    let (open, close) = market_hours();
    time < open || time >= close
}

pub fn milliseconds_to_date(ms: i64) -> DateEST {
    let seconds = ms / 1000;
    let naive_date = NaiveDateTime::from_timestamp(seconds, 0);
    let est = FixedOffset::west(4 * HOURS);
    DateTime::<FixedOffset>::from_utc(naive_date, est)
}

pub fn weeks_ago(weeks: i64) -> Date {
    current_date() - Duration::weeks(weeks)
}

pub fn days_ago(days: i64) -> Date {
    current_date() - self::days(days)
}

pub fn days(days: i64) -> Duration {
    Duration::days(days)
}
