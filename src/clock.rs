pub use chrono::Duration;
use chrono::{Date, DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

pub type LocalDateTime = DateTime<Local>;
pub type Time = NaiveTime;
pub type DateWithoutTZ = NaiveDate;
pub type LocalDate = Date<Local>;

pub fn current_date() -> DateWithoutTZ {
    Local::today().naive_local()
}

pub fn milliseconds_to_date(ms: i64) -> LocalDateTime {
    let seconds = ms / 1000;
    let naive_datetime = NaiveDateTime::from_timestamp(seconds, 0);
    Local.from_local_datetime(&naive_datetime).single().unwrap()
}

pub fn parse_datetime(datetime: &str) -> LocalDateTime {
    let naive_dt = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S");
    Local
        .from_local_datetime(&naive_dt.unwrap())
        .single()
        .unwrap()
}

pub fn days_ago(days: i64) -> DateWithoutTZ {
    current_date() - self::days(days)
}

pub fn days(days: i64) -> Duration {
    Duration::days(days)
}

pub fn datetime(y: i32, month: u32, d: u32, h: u32, m: u32, s: u32) -> LocalDateTime {
    Local.ymd(y, month, d).and_hms(h, m, s)
}

pub fn day_of_week(date: LocalDate) -> i32 {
    date.format("%u").to_string().parse().unwrap()
}
