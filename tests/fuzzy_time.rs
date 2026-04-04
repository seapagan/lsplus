use lsplus::utils::fuzzy_time::fuzzy_time;
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

    let time = now.checked_sub(Duration::from_secs(59)).unwrap();
    assert_eq!(fuzzy_time(time), "59 seconds ago");

    let time = now.checked_sub(Duration::from_secs(60)).unwrap();
    assert_eq!(fuzzy_time(time), "1 minute ago");

    let time = now.checked_sub(Duration::from_secs(119)).unwrap();
    assert_eq!(fuzzy_time(time), "1 minute ago");

    let time = now.checked_sub(Duration::from_secs(120)).unwrap();
    assert_eq!(fuzzy_time(time), "2 minutes ago");

    let time = now.checked_sub(Duration::from_secs(23 * 60 * 60)).unwrap();
    assert_eq!(fuzzy_time(time), "23 hours ago");

    let time = now.checked_sub(Duration::from_secs(24 * 60 * 60)).unwrap();
    assert_eq!(fuzzy_time(time), "yesterday");

    let time = now
        .checked_sub(Duration::from_secs(6 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "6 days ago");

    let time = now
        .checked_sub(Duration::from_secs(7 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "last week");
}

#[test]
fn test_fuzzy_time_month_boundaries() {
    let now = SystemTime::now();

    let time = now
        .checked_sub(Duration::from_secs(29 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "4 weeks ago");

    let time = now
        .checked_sub(Duration::from_secs(30 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "last month");

    let time = now
        .checked_sub(Duration::from_secs(45 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "last month");

    let time = now
        .checked_sub(Duration::from_secs(60 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "2 months ago");
}

#[test]
fn test_fuzzy_time_year_boundaries() {
    let now = SystemTime::now();

    let time = now
        .checked_sub(Duration::from_secs(364 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "12 months ago");

    let time = now
        .checked_sub(Duration::from_secs(365 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "last year");

    let time = now
        .checked_sub(Duration::from_secs(729 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "last year");

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

    let time = now
        .checked_sub(Duration::from_secs(13 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "last week");

    let time = now
        .checked_sub(Duration::from_secs(14 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "2 weeks ago");

    let time = now
        .checked_sub(Duration::from_secs(20 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "2 weeks ago");

    let time = now
        .checked_sub(Duration::from_secs(21 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "3 weeks ago");
}

#[test]
fn test_fuzzy_time_month_edge_cases() {
    let now = SystemTime::now();

    let time = now
        .checked_sub(Duration::from_secs(59 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "last month");

    let time = now
        .checked_sub(Duration::from_secs(61 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "2 months ago");

    let time = now
        .checked_sub(Duration::from_secs(89 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "2 months ago");

    let time = now
        .checked_sub(Duration::from_secs(90 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "3 months ago");
}

#[test]
fn test_fuzzy_time_very_old_files() {
    let now = SystemTime::now();

    let time = now
        .checked_sub(Duration::from_secs(2 * 365 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "2 years ago");

    let time = now
        .checked_sub(Duration::from_secs(5 * 365 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "5 years ago");

    let time = now
        .checked_sub(Duration::from_secs(10 * 365 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "10 years ago");

    let time = now
        .checked_sub(Duration::from_secs(20 * 365 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "20 years ago");

    let time = now
        .checked_sub(Duration::from_secs(50 * 365 * 24 * 60 * 60))
        .unwrap();
    assert_eq!(fuzzy_time(time), "50 years ago");
}

#[test]
fn test_fuzzy_time_hour_edge_cases() {
    let now = SystemTime::now();

    let time = now.checked_sub(Duration::from_secs(59 * 60)).unwrap();
    assert_eq!(fuzzy_time(time), "59 minutes ago");

    let time = now.checked_sub(Duration::from_secs(61 * 60)).unwrap();
    assert_eq!(fuzzy_time(time), "1 hour ago");

    let time = now.checked_sub(Duration::from_secs(119 * 60)).unwrap();
    assert_eq!(fuzzy_time(time), "1 hour ago");

    let time = now.checked_sub(Duration::from_secs(120 * 60)).unwrap();
    assert_eq!(fuzzy_time(time), "2 hours ago");
}

#[test]
fn test_fuzzy_time_uses_zero_seconds_for_future_times() {
    let future = SystemTime::now()
        .checked_add(Duration::from_secs(60))
        .unwrap();

    assert_eq!(fuzzy_time(future), "0 seconds ago");
}
