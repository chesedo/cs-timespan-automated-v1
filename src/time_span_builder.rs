//! Fluent, Rust-idiomatic multi-component constructor for [`TimeSpan`].
//!
//! This replaces an earlier DHMS constructor family (`TimeSpan::from_hms`/
//! `from_dhms`/`from_dhms_milli`/`from_dhms_micro`) and `*_parts` factories
//! (`from_days_parts`, `from_hours_parts`, ...), which ported C#'s multi-component
//! `TimeSpan` constructor overloads and `FromX(int, int, ...)` static factory
//! overloads faithfully, including their per-overload field-type inconsistency (the
//! DHMS family was all `i32`; the `*_parts` family mixed `i32` days/hours with `i64`
//! minutes/seconds/milliseconds/microseconds, mirroring C#'s own per-overload
//! inconsistency). [`TimeSpanBuilder`] instead uses `i64` uniformly for every field —
//! the widest type either family used per field, so nothing was lost by widening
//! (`i32` values convert losslessly via `i64::from(...)`).

use crate::{TimeSpan, TimeSpanError};

/// Cf. TimeSpan.cs#L210-L211 (internal `MinMicroseconds`/`MaxMicroseconds`). Bounds for
/// [`TimeSpanBuilder::build`]'s summed microsecond total; meaningful only in service of
/// that range check, so kept local here rather than on [`TimeSpan`] itself.
const MIN_MICROSECONDS: i64 = i64::MIN / TimeSpan::TICKS_PER_MICROSECOND;
const MAX_MICROSECONDS: i64 = i64::MAX / TimeSpan::TICKS_PER_MICROSECOND;

/// Fluent builder for [`TimeSpan`], covering the day/hour/minute/second/
/// millisecond/microsecond component space, with `i64` fields throughout.
///
/// Construct one via [`TimeSpan::builder`]; every field defaults to `0`, so
/// `TimeSpan::builder().build()` yields [`TimeSpan::ZERO`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TimeSpanBuilder {
    days: i64,
    hours: i64,
    minutes: i64,
    seconds: i64,
    milliseconds: i64,
    microseconds: i64,
}

impl TimeSpanBuilder {
    /// Sets the `days` component. Unset (default) is `0`.
    #[must_use]
    pub fn days(mut self, value: i64) -> Self {
        self.days = value;
        self
    }

    /// Sets the `hours` component. Unset (default) is `0`.
    #[must_use]
    pub fn hours(mut self, value: i64) -> Self {
        self.hours = value;
        self
    }

    /// Sets the `minutes` component. Unset (default) is `0`.
    #[must_use]
    pub fn minutes(mut self, value: i64) -> Self {
        self.minutes = value;
        self
    }

    /// Sets the `seconds` component. Unset (default) is `0`.
    #[must_use]
    pub fn seconds(mut self, value: i64) -> Self {
        self.seconds = value;
        self
    }

    /// Sets the `milliseconds` component. Unset (default) is `0`.
    #[must_use]
    pub fn milliseconds(mut self, value: i64) -> Self {
        self.milliseconds = value;
        self
    }

    /// Sets the `microseconds` component. Unset (default) is `0`.
    #[must_use]
    pub fn microseconds(mut self, value: i64) -> Self {
        self.microseconds = value;
        self
    }

    /// Sums all six components into a validated [`TimeSpan`]. Widens to `i128`
    /// while summing so the addition itself can never overflow, matching
    /// `ArgumentOutOfRangeException`'s out-of-range check.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if the combined `days`/`hours`/`minutes`/
    /// `seconds`/`milliseconds`/`microseconds` value falls outside the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L292-L306 (6-arg constructor body)
    pub fn build(self) -> Result<TimeSpan, TimeSpanError> {
        let total_microseconds = i128::from(self.days) * i128::from(TimeSpan::MICROSECONDS_PER_DAY)
            + i128::from(self.hours) * i128::from(TimeSpan::MICROSECONDS_PER_HOUR)
            + i128::from(self.minutes) * i128::from(TimeSpan::MICROSECONDS_PER_MINUTE)
            + i128::from(self.seconds) * i128::from(TimeSpan::MICROSECONDS_PER_SECOND)
            + i128::from(self.milliseconds) * i128::from(TimeSpan::MICROSECONDS_PER_MILLISECOND)
            + i128::from(self.microseconds);

        if total_microseconds > i128::from(MAX_MICROSECONDS)
            || total_microseconds < i128::from(MIN_MICROSECONDS)
        {
            return Err(TimeSpanError::Overflow);
        }

        #[allow(
            clippy::cast_possible_truncation,
            reason = "total_microseconds is bounds-checked against MAX_MICROSECONDS/MIN_MICROSECONDS \
                      (i64::MAX/MIN divided by TICKS_PER_MICROSECOND) above, so it fits in i64"
        )]
        let ticks = total_microseconds as i64 * TimeSpan::TICKS_PER_MICROSECOND;
        Ok(TimeSpan::from_ticks(ticks))
    }
}
