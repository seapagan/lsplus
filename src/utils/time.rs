//! Shared duration constants for age bucketing.

use std::time::Duration;

/// One calendar-ish day.
pub(crate) const DAY: Duration = Duration::from_secs(24 * 60 * 60);
/// Seven days.
pub(crate) const WEEK: Duration = Duration::from_secs(7 * DAY.as_secs());
/// Thirty days, used as the month bucket for display gradients.
pub(crate) const MONTH: Duration = Duration::from_secs(30 * DAY.as_secs());
/// Three hundred sixty-five days.
pub(crate) const YEAR: Duration = Duration::from_secs(365 * DAY.as_secs());
