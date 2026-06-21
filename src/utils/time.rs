use std::time::Duration;

pub(crate) const DAY: Duration = Duration::from_secs(24 * 60 * 60);
pub(crate) const WEEK: Duration = Duration::from_secs(7 * DAY.as_secs());
pub(crate) const MONTH: Duration = Duration::from_secs(30 * DAY.as_secs());
pub(crate) const YEAR: Duration = Duration::from_secs(365 * DAY.as_secs());
