pub use chrono::Duration;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

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

pub fn outside_market_hours(datetime: DateEST) -> bool {
    let (open, close) = market_hours();
    let time = datetime.time();
    let day_of_week: i32 = datetime.date().format("%u").to_string().parse().unwrap();
    time < open || time >= close || day_of_week > 5
}

pub fn milliseconds_to_date(ms: i64) -> DateEST {
    let seconds = ms / 1000;
    let naive_date = NaiveDateTime::from_timestamp(seconds, 0);
    let est = FixedOffset::west(4 * HOURS);
    DateTime::<FixedOffset>::from_utc(naive_date, est)
}

pub fn days_ago(days: i64) -> Date {
    current_date() - self::days(days)
}

pub fn days(days: i64) -> Duration {
    Duration::days(days)
}

pub fn datetime(y: i32, month: u32, d: u32, h: u32, m: u32, s: u32) -> DateTime<FixedOffset> {
    FixedOffset::west(4 * HOURS)
        .ymd(y, month, d)
        .and_hms(h, m, s)
}

#[cfg(test)]
mod tests {
    use super::{datetime, outside_market_hours, HOURS};
    use chrono::{FixedOffset, TimeZone};

    #[test]
    fn market_hours() {
        let time = datetime(2020, 9, 29, 9, 30, 00);
        assert_eq!(outside_market_hours(time), false);
    }

    #[test]
    fn pre_market_hours() {
        let time = FixedOffset::west(4 * HOURS)
            .ymd(2020, 9, 29)
            .and_hms(9, 29, 59);
        assert_eq!(outside_market_hours(time), true);
    }

    #[test]
    fn post_market_hours() {
        let time = FixedOffset::west(4 * HOURS)
            .ymd(2020, 9, 29)
            .and_hms(16, 0, 0);
        assert_eq!(outside_market_hours(time), true);
    }

    #[test]
    fn saturday_is_outside_market_hours() {
        let time = FixedOffset::west(4 * HOURS)
            .ymd(2020, 9, 26)
            .and_hms(10, 0, 0);
        assert_eq!(outside_market_hours(time), true);
    }

    #[test]
    fn sunday_is_outside_market_hours() {
        let time = FixedOffset::west(4 * HOURS)
            .ymd(2020, 9, 26)
            .and_hms(10, 0, 0);
        assert_eq!(outside_market_hours(time), true);
    }
}
