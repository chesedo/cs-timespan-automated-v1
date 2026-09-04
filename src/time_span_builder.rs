//! Fluent, Rust-idiomatic multi-component constructor for [`TimeSpan`].
//!
//! Covers the same ground as C#'s multi-component `TimeSpan` constructor overloads
//! and `FromX(int, int, ...)` static factory overloads — day/hour/minute/second/
//! millisecond/microsecond components — as one builder instead of several
//! fixed-arity overloads. Uses `i64` uniformly for every field rather than mirroring
//! each C# overload's own per-parameter type (some are `int`, others `long`,
//! depending on which overload); `i32`-sized values still convert losslessly via
//! `i64::from(...)`, so nothing is lost by the wider, uniform type.

use crate::{TimeSpan, TimeSpanError};

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

    /// Sums all six components into a validated [`TimeSpan`], via the same
    /// `i128`-widened-sum-then-range-check logic as the DHMS constructor family
    /// (`TimeSpan::dhms_to_ticks`).
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if the combined `days`/`hours`/`minutes`/
    /// `seconds`/`milliseconds`/`microseconds` value falls outside the range
    /// representable by `TimeSpan`.
    pub fn build(self) -> Result<TimeSpan, TimeSpanError> {
        let ticks = TimeSpan::dhms_to_ticks(
            self.days,
            self.hours,
            self.minutes,
            self.seconds,
            self.milliseconds,
            self.microseconds,
        )?;
        Ok(TimeSpan::from_ticks(ticks))
    }
}
