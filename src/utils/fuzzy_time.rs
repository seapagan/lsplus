//! Human-readable relative timestamp formatting.
//!
//! Past timestamps are rendered as phrases such as `2 hours ago`, `yesterday`,
//! or `last month`; future timestamps are rendered as `in ...` phrases.

use std::fmt;
use std::time::{Duration, SystemTime};

use crate::utils::time::{DAY, MONTH, WEEK, YEAR};

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

/// Format one relative-time unit with simple pluralization.
fn format_unit(unit: &str, n: u64) -> String {
    format!("{} {}{}", n, unit, if n == 1 { "" } else { "s" })
}

impl fmt::Display for FuzzyTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// Format a future timestamp offset.
fn format_future_time(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = seconds / DAY.as_secs();
    let weeks = seconds / WEEK.as_secs();
    let months = seconds / MONTH.as_secs();
    let years = seconds / YEAR.as_secs();

    let (unit, n) = match seconds {
        s if s < 60 => ("second", s),
        _ if minutes < 60 => ("minute", minutes),
        _ if hours < 24 => ("hour", hours),
        _ if duration < WEEK => ("day", days),
        _ if duration < MONTH => ("week", weeks),
        _ if duration < YEAR => ("month", months),
        _ => ("year", years),
    };

    format!("in {}", format_unit(unit, n))
}

/// Bucket a past timestamp offset into the phrase used for display.
fn get_fuzzy_time(duration: Duration) -> FuzzyTime {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = seconds / DAY.as_secs();
    let weeks = seconds / WEEK.as_secs();
    let months = seconds / MONTH.as_secs();
    let years = seconds / YEAR.as_secs();

    match seconds {
        s if s < 60 => FuzzyTime::SecondsAgo(s),
        _ if minutes < 60 => FuzzyTime::MinutesAgo(minutes),
        _ if hours < 24 => FuzzyTime::HoursAgo(hours),
        _ if days == 1 => FuzzyTime::Yesterday,
        _ if duration < WEEK => FuzzyTime::DaysAgo(days),
        _ if duration < 2 * WEEK => FuzzyTime::LastWeek,
        _ if duration < MONTH => FuzzyTime::WeeksAgo(weeks),
        _ if duration < 2 * MONTH => FuzzyTime::LastMonth,
        _ if duration < YEAR => FuzzyTime::MonthsAgo(months),
        _ if duration < 2 * YEAR => FuzzyTime::LastYear,
        _ => FuzzyTime::YearsAgo(years),
    }
}

/// Return a human-readable relative time string for a system timestamp.
pub fn fuzzy_time(time: SystemTime) -> String {
    let now = SystemTime::now();
    match now.duration_since(time) {
        Ok(duration) => get_fuzzy_time(duration).to_string(),
        Err(error) => format_future_time(error.duration()),
    }
}
