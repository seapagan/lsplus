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

    fn get_test_time(seconds_ago: u64) -> SystemTime {
        SystemTime::now()
            .checked_sub(Duration::from_secs(seconds_ago))
            .unwrap()
    }

    #[test]
    fn test_fuzzy_time_seconds() {
        let time = get_test_time(30);
        assert_eq!(fuzzy_time(time), "30 seconds ago");
    }

    #[test]
    fn test_fuzzy_time_minutes() {
        let time = get_test_time(5 * 60);
        assert_eq!(fuzzy_time(time), "5 minutes ago");

        let time = get_test_time(60);
        assert_eq!(fuzzy_time(time), "1 minute ago");
    }

    #[test]
    fn test_fuzzy_time_hours() {
        let time = get_test_time(2 * 60 * 60);
        assert_eq!(fuzzy_time(time), "2 hours ago");

        let time = get_test_time(60 * 60);
        assert_eq!(fuzzy_time(time), "1 hour ago");
    }

    #[test]
    fn test_fuzzy_time_days() {
        let time = get_test_time(2 * 24 * 60 * 60);
        assert_eq!(fuzzy_time(time), "2 days ago");

        let time = get_test_time(24 * 60 * 60);
        assert_eq!(fuzzy_time(time), "yesterday");
    }

    #[test]
    fn test_fuzzy_time_weeks() {
        let time = get_test_time(2 * 7 * 24 * 60 * 60);
        assert_eq!(fuzzy_time(time), "2 weeks ago");

        let time = get_test_time(7 * 24 * 60 * 60);
        assert_eq!(fuzzy_time(time), "last week");
    }

    #[test]
    fn test_fuzzy_time_months() {
        let time = get_test_time(60 * 24 * 60 * 60);
        assert_eq!(fuzzy_time(time), "2 months ago");

        let time = get_test_time(32 * 24 * 60 * 60);
        assert_eq!(fuzzy_time(time), "last month");
    }

    #[test]
    fn test_fuzzy_time_years() {
        let time = get_test_time(2 * 365 * 24 * 60 * 60);
        assert_eq!(fuzzy_time(time), "2 years ago");

        let time = get_test_time(366 * 24 * 60 * 60);
        assert_eq!(fuzzy_time(time), "last year");
    }

    #[test]
    fn test_fuzzy_time_boundary_cases() {
        let now = SystemTime::now();

        // Test exactly 59 seconds
        let time = now.checked_sub(Duration::from_secs(59)).unwrap();
        assert_eq!(fuzzy_time(time), "59 seconds ago");

        // Test exactly 60 seconds (should show as 1 minute)
        let time = now.checked_sub(Duration::from_secs(60)).unwrap();
        assert_eq!(fuzzy_time(time), "1 minute ago");

        // Test 1 minute and 59 seconds (should show as 1 minute)
        let time = now.checked_sub(Duration::from_secs(119)).unwrap();
        assert_eq!(fuzzy_time(time), "1 minute ago");

        // Test exactly 2 minutes
        let time = now.checked_sub(Duration::from_secs(120)).unwrap();
        assert_eq!(fuzzy_time(time), "2 minutes ago");

        // Test exactly 23 hours
        let time = now.checked_sub(Duration::from_secs(23 * 60 * 60)).unwrap();
        assert_eq!(fuzzy_time(time), "23 hours ago");

        // Test exactly 24 hours (should show as "yesterday")
        let time = now.checked_sub(Duration::from_secs(24 * 60 * 60)).unwrap();
        assert_eq!(fuzzy_time(time), "yesterday");

        // Test 6 days
        let time = now
            .checked_sub(Duration::from_secs(6 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "6 days ago");

        // Test 7 days (should show as "last week")
        let time = now
            .checked_sub(Duration::from_secs(7 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "last week");
    }

    #[test]
    fn test_fuzzy_time_month_boundaries() {
        let now = SystemTime::now();

        // Test 29 days (should still show weeks)
        let time = now
            .checked_sub(Duration::from_secs(29 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "4 weeks ago");

        // Test 30 days (should show "last month")
        let time = now
            .checked_sub(Duration::from_secs(30 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "last month");

        // Test 45 days (should show "last month")
        let time = now
            .checked_sub(Duration::from_secs(45 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "last month");

        // Test 60 days (should show "2 months ago")
        let time = now
            .checked_sub(Duration::from_secs(60 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "2 months ago");
    }

    #[test]
    fn test_fuzzy_time_year_boundaries() {
        let now = SystemTime::now();

        // Test 364 days (should show months)
        let time = now
            .checked_sub(Duration::from_secs(364 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "12 months ago");

        // Test 365 days (should show "last year")
        let time = now
            .checked_sub(Duration::from_secs(365 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "last year");

        // Test 729 days (should still show "last year")
        let time = now
            .checked_sub(Duration::from_secs(729 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "last year");

        // Test 730 days (should show "2 years ago")
        let time = now
            .checked_sub(Duration::from_secs(730 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "2 years ago");
    }

    #[test]
    fn test_fuzzy_time_zero_duration() {
        let now = SystemTime::now();
        assert_eq!(fuzzy_time(now), "0 seconds ago");
    }

    #[test]
    fn test_fuzzy_time_week_boundaries() {
        let now = SystemTime::now();

        // Test 13 days (should still show "last week")
        let time = now
            .checked_sub(Duration::from_secs(13 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "last week");

        // Test 14 days (should show "2 weeks ago")
        let time = now
            .checked_sub(Duration::from_secs(14 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "2 weeks ago");

        // Test 20 days (should show "2 weeks ago")
        let time = now
            .checked_sub(Duration::from_secs(20 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "2 weeks ago");

        // Test 21 days (should show "3 weeks ago")
        let time = now
            .checked_sub(Duration::from_secs(21 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "3 weeks ago");
    }

    #[test]
    fn test_fuzzy_time_month_edge_cases() {
        let now = SystemTime::now();

        // Test 59 days (should still show "last month")
        let time = now
            .checked_sub(Duration::from_secs(59 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "last month");

        // Test 61 days (should show "2 months ago")
        let time = now
            .checked_sub(Duration::from_secs(61 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "2 months ago");

        // Test 89 days (should show "2 months ago")
        let time = now
            .checked_sub(Duration::from_secs(89 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "2 months ago");

        // Test 90 days (should show "3 months ago")
        let time = now
            .checked_sub(Duration::from_secs(90 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "3 months ago");
    }

    #[test]
    fn test_fuzzy_time_very_old_files() {
        let now = SystemTime::now();

        // Test 2 years exactly
        let time = now
            .checked_sub(Duration::from_secs(2 * 365 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "2 years ago");

        // Test 5 years
        let time = now
            .checked_sub(Duration::from_secs(5 * 365 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "5 years ago");

        // Test 10 years
        let time = now
            .checked_sub(Duration::from_secs(10 * 365 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "10 years ago");

        // Test 20 years
        let time = now
            .checked_sub(Duration::from_secs(20 * 365 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "20 years ago");

        // Test 50 years (like old Unix timestamps)
        let time = now
            .checked_sub(Duration::from_secs(50 * 365 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(fuzzy_time(time), "50 years ago");
    }

    #[test]
    fn test_fuzzy_time_hour_edge_cases() {
        let now = SystemTime::now();

        // Test 59 minutes (should still show minutes)
        let time = now.checked_sub(Duration::from_secs(59 * 60)).unwrap();
        assert_eq!(fuzzy_time(time), "59 minutes ago");

        // Test 61 minutes (should show "1 hour ago")
        let time = now.checked_sub(Duration::from_secs(61 * 60)).unwrap();
        assert_eq!(fuzzy_time(time), "1 hour ago");

        // Test 119 minutes (should show "1 hour ago")
        let time = now.checked_sub(Duration::from_secs(119 * 60)).unwrap();
        assert_eq!(fuzzy_time(time), "1 hour ago");

        // Test 120 minutes (should show "2 hours ago")
        let time = now.checked_sub(Duration::from_secs(120 * 60)).unwrap();
        assert_eq!(fuzzy_time(time), "2 hours ago");
    }
}
