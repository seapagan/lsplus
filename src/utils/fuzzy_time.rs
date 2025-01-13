use std::fmt;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
enum FuzzyTime {
    SecondsAgo(u64),
    MinutesAgo(u64),
    HoursAgo(u64),
    DaysAgo(u64),
    WeeksAgo(u64),
    MonthsAgo(u64),
    YearsAgo(u64),
    LastWeek,
    LastMonth,
    LastYear,
    Yesterday,
}

impl fmt::Display for FuzzyTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn format_unit(unit: &str, n: u64) -> String {
            format!("{} {}{}", n, unit, if n == 1 { "" } else { "s" })
        }

        match self {
            FuzzyTime::SecondsAgo(n) => {
                write!(f, "{} ago", format_unit("second", *n))
            }
            FuzzyTime::MinutesAgo(n) => {
                write!(f, "{} ago", format_unit("minute", *n))
            }
            FuzzyTime::HoursAgo(n) => {
                write!(f, "{} ago", format_unit("hour", *n))
            }
            FuzzyTime::DaysAgo(n) => {
                write!(f, "{} ago", format_unit("day", *n))
            }
            FuzzyTime::WeeksAgo(n) => {
                write!(f, "{} ago", format_unit("week", *n))
            }
            FuzzyTime::MonthsAgo(n) => {
                write!(f, "{} ago", format_unit("month", *n))
            }
            FuzzyTime::YearsAgo(n) => {
                write!(f, "{} ago", format_unit("year", *n))
            }
            FuzzyTime::LastWeek => write!(f, "last week"),
            FuzzyTime::LastMonth => write!(f, "last month"),
            FuzzyTime::LastYear => write!(f, "last year"),
            FuzzyTime::Yesterday => write!(f, "yesterday"),
        }
    }
}

fn get_fuzzy_time(duration: Duration) -> FuzzyTime {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let weeks = days / 7;
    let months = days / 30;
    let years = days / 365;

    match seconds {
        s if s < 60 => FuzzyTime::SecondsAgo(s),
        _ if minutes < 60 => FuzzyTime::MinutesAgo(minutes),
        _ if hours < 24 => FuzzyTime::HoursAgo(hours),
        _ if days == 1 => FuzzyTime::Yesterday,
        _ if days < 7 => FuzzyTime::DaysAgo(days),
        _ if days < 14 => FuzzyTime::LastWeek,
        _ if days < 30 => FuzzyTime::WeeksAgo(weeks),
        _ if days < 60 => FuzzyTime::LastMonth,
        _ if days < 365 => FuzzyTime::MonthsAgo(months),
        _ if days < 730 => FuzzyTime::LastYear,
        _ => FuzzyTime::YearsAgo(years),
    }
}

pub fn fuzzy_time(time: SystemTime) -> String {
    let now = SystemTime::now();
    let duration = now
        .duration_since(time)
        .unwrap_or_else(|_| Duration::from_secs(0));
    get_fuzzy_time(duration).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_fuzzy_time_seconds_ago() {
        let now = SystemTime::now();
        let ten_seconds_ago = now - Duration::new(10, 0);
        assert_eq!(fuzzy_time(ten_seconds_ago), "10 seconds ago");
    }

    #[test]
    fn test_fuzzy_time_minutes_ago() {
        let now = SystemTime::now();
        let five_minutes_ago = now - Duration::new(5 * 60, 0);
        assert_eq!(fuzzy_time(five_minutes_ago), "5 minutes ago");
    }

    #[test]
    fn test_fuzzy_time_hours_ago() {
        let now = SystemTime::now();
        let three_hours_ago = now - Duration::new(3 * 60 * 60, 0);
        assert_eq!(fuzzy_time(three_hours_ago), "3 hours ago");
    }

    #[test]
    fn test_fuzzy_time_yesterday() {
        let now = SystemTime::now();
        let one_day_ago = now - Duration::new(24 * 60 * 60, 0);
        assert_eq!(fuzzy_time(one_day_ago), "yesterday");
    }

    #[test]
    fn test_fuzzy_time_days_ago() {
        let now = SystemTime::now();
        let four_days_ago = now - Duration::new(4 * 24 * 60 * 60, 0);
        assert_eq!(fuzzy_time(four_days_ago), "4 days ago");
    }

    #[test]
    fn test_fuzzy_time_last_week() {
        let now = SystemTime::now();
        let ten_days_ago = now - Duration::new(10 * 24 * 60 * 60, 0);
        assert_eq!(fuzzy_time(ten_days_ago), "last week");
    }

    #[test]
    fn test_fuzzy_time_weeks_ago() {
        let now = SystemTime::now();
        let three_weeks_ago = now - Duration::new(3 * 7 * 24 * 60 * 60, 0);
        assert_eq!(fuzzy_time(three_weeks_ago), "3 weeks ago");
    }

    #[test]
    fn test_fuzzy_time_last_month() {
        let now = SystemTime::now();
        let forty_five_days_ago = now - Duration::new(45 * 24 * 60 * 60, 0);
        assert_eq!(fuzzy_time(forty_five_days_ago), "last month");
    }

    #[test]
    fn test_fuzzy_time_months_ago() {
        let now = SystemTime::now();
        let five_months_ago = now - Duration::new(5 * 30 * 24 * 60 * 60, 0);
        assert_eq!(fuzzy_time(five_months_ago), "5 months ago");
    }

    #[test]
    fn test_fuzzy_time_last_year() {
        let now = SystemTime::now();
        let four_hundred_days_ago = now - Duration::new(400 * 24 * 60 * 60, 0);
        assert_eq!(fuzzy_time(four_hundred_days_ago), "last year");
    }

    #[test]
    fn test_fuzzy_time_years_ago() {
        let now = SystemTime::now();
        let two_years_ago = now - Duration::new(2 * 365 * 24 * 60 * 60, 0);
        assert_eq!(fuzzy_time(two_years_ago), "2 years ago");
    }
}
