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
        match self {
            FuzzyTime::SecondsAgo(n) => {
                if *n == 1 {
                    write!(f, "1 second ago")
                } else {
                    write!(f, "{} seconds ago", n)
                }
            }
            FuzzyTime::MinutesAgo(n) => {
                if *n == 1 {
                    write!(f, "1 minute ago")
                } else {
                    write!(f, "{} minutes ago", n)
                }
            }
            FuzzyTime::HoursAgo(n) => {
                if *n == 1 {
                    write!(f, "1 hour ago")
                } else {
                    write!(f, "{} hours ago", n)
                }
            }
            FuzzyTime::DaysAgo(n) => {
                if *n == 1 {
                    write!(f, "yesterday")
                } else {
                    write!(f, "{} days ago", n)
                }
            }
            FuzzyTime::WeeksAgo(n) => {
                if *n == 1 {
                    write!(f, "1 week ago")
                } else {
                    write!(f, "{} weeks ago", n)
                }
            }
            FuzzyTime::MonthsAgo(n) => {
                if *n == 1 {
                    write!(f, "1 month ago")
                } else {
                    write!(f, "{} months ago", n)
                }
            }
            FuzzyTime::YearsAgo(n) => {
                if *n == 1 {
                    write!(f, "1 year ago")
                } else {
                    write!(f, "{} years ago", n)
                }
            }
            FuzzyTime::LastWeek => write!(f, "last week"),
            FuzzyTime::LastMonth => write!(f, "last month"),
            FuzzyTime::LastYear => write!(f, "last year"),
            FuzzyTime::Yesterday => write!(f, "yesterday"),
        }
    }
}

pub fn fuzzy_time(time: SystemTime) -> String {
    let now = SystemTime::now();
    let duration = now
        .duration_since(time)
        .unwrap_or_else(|_| Duration::from_secs(0));

    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let weeks = days / 7;
    let months = days / 30;
    let years = days / 365;

    let fuzzy_time = if seconds < 60 {
        FuzzyTime::SecondsAgo(seconds)
    } else if minutes < 60 {
        FuzzyTime::MinutesAgo(minutes)
    } else if hours < 24 {
        FuzzyTime::HoursAgo(hours)
    } else if days == 1 {
        FuzzyTime::Yesterday
    } else if days < 7 {
        FuzzyTime::DaysAgo(days)
    } else if days < 14 {
        FuzzyTime::LastWeek
    } else if days < 30 {
        FuzzyTime::WeeksAgo(weeks)
    } else if days < 60 {
        FuzzyTime::LastMonth
    } else if days < 365 {
        FuzzyTime::MonthsAgo(months)
    } else if days < 730 {
        FuzzyTime::LastYear
    } else {
        FuzzyTime::YearsAgo(years)
    };

    fuzzy_time.to_string()
}
