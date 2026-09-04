use std::str::FromStr;

use crate::error::TimeSpanError;
use crate::time_span_builder::TimeSpanBuilder;

/// A duration of time, represented as a number of 100-nanosecond ticks.
///
/// Cf. TimeSpan.cs#L11-L20
///
/// ```
/// use cs_timespan_automated_v1::TimeSpan;
///
/// let ts = TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR);
/// assert_eq!(ts.hours(), 1);
/// assert_eq!(ts.ticks(), TimeSpan::TICKS_PER_HOUR);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeSpan {
    ticks: i64,
}

/// Controls how [`TimeSpan::parse_exact`]'s custom-format-string path interprets a parsed
/// value's sign.
///
/// C#'s `[Flags] enum TimeSpanStyles { None = 0, AssumeNegative = 1 }` is a bitflag type,
/// but with only one real flag it never combines with anything — represented here as an
/// ordinary 2-variant enum instead. An out-of-range value (C#'s `TimeSpanStyles.None - 1` /
/// `TimeSpanStyles.AssumeNegative + 1`, which `ParseExact` rejects with `ArgumentException`)
/// is simply unrepresentable in Rust's type system, so that validation has no Rust
/// equivalent to port — see [`crate::TimeSpanError`]'s doc comment, which already notes
/// this.
///
/// Only `parse_exact`'s custom-format-string path consults this at all: C#'s single-letter
/// standard formats (`"c"`/`"t"`/`"T"`/`"g"`/`"G"`) ignore `TimeSpanStyles` entirely
/// (`TimeSpanTests.cs`'s `ParseExact` test only asserts `AssumeNegative` behavior when
/// `format` isn't one of those five, and TimeSpanParse.cs#L1237-1241's dispatch never
/// passes `styles` through to the standard-format algorithms at all).
///
/// Cf. TimeSpanStyles.cs (`System.Globalization.TimeSpanStyles`)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TimeSpanStyles {
    /// Cf. TimeSpanStyles.cs's `None`.
    #[default]
    None,
    /// A leading `-` is not expected in the input; instead, the parsed magnitude is negated
    /// unconditionally.
    ///
    /// Cf. TimeSpanStyles.cs's `AssumeNegative`.
    AssumeNegative,
}

impl TimeSpan {
    // Cf. TimeSpan.cs#L37-L205
    pub const NANOSECONDS_PER_TICK: i64 = 100;
    pub const TICKS_PER_MICROSECOND: i64 = 10;
    pub const TICKS_PER_MILLISECOND: i64 = Self::TICKS_PER_MICROSECOND * 1000;
    pub const TICKS_PER_SECOND: i64 = Self::TICKS_PER_MILLISECOND * 1000;
    pub const TICKS_PER_MINUTE: i64 = Self::TICKS_PER_SECOND * 60;
    pub const TICKS_PER_HOUR: i64 = Self::TICKS_PER_MINUTE * 60;
    pub const TICKS_PER_DAY: i64 = Self::TICKS_PER_HOUR * 24;

    pub const MICROSECONDS_PER_MILLISECOND: i64 =
        Self::TICKS_PER_MILLISECOND / Self::TICKS_PER_MICROSECOND;
    pub const MICROSECONDS_PER_SECOND: i64 = Self::TICKS_PER_SECOND / Self::TICKS_PER_MICROSECOND;
    pub const MICROSECONDS_PER_MINUTE: i64 = Self::TICKS_PER_MINUTE / Self::TICKS_PER_MICROSECOND;
    pub const MICROSECONDS_PER_HOUR: i64 = Self::TICKS_PER_HOUR / Self::TICKS_PER_MICROSECOND;
    pub const MICROSECONDS_PER_DAY: i64 = Self::TICKS_PER_DAY / Self::TICKS_PER_MICROSECOND;

    pub const MILLISECONDS_PER_SECOND: i64 = Self::TICKS_PER_SECOND / Self::TICKS_PER_MILLISECOND;
    pub const MILLISECONDS_PER_MINUTE: i64 = Self::TICKS_PER_MINUTE / Self::TICKS_PER_MILLISECOND;
    pub const MILLISECONDS_PER_HOUR: i64 = Self::TICKS_PER_HOUR / Self::TICKS_PER_MILLISECOND;
    pub const MILLISECONDS_PER_DAY: i64 = Self::TICKS_PER_DAY / Self::TICKS_PER_MILLISECOND;

    pub const SECONDS_PER_MINUTE: i64 = Self::TICKS_PER_MINUTE / Self::TICKS_PER_SECOND;
    pub const SECONDS_PER_HOUR: i64 = Self::TICKS_PER_HOUR / Self::TICKS_PER_SECOND;
    pub const SECONDS_PER_DAY: i64 = Self::TICKS_PER_DAY / Self::TICKS_PER_SECOND;

    pub const MINUTES_PER_HOUR: i64 = Self::TICKS_PER_HOUR / Self::TICKS_PER_MINUTE;
    pub const MINUTES_PER_DAY: i64 = Self::TICKS_PER_DAY / Self::TICKS_PER_MINUTE;

    #[allow(
        clippy::cast_possible_truncation,
        reason = "TICKS_PER_DAY / TICKS_PER_HOUR is the compile-time constant 24, far within i32's \
                  range"
    )]
    pub const HOURS_PER_DAY: i32 = (Self::TICKS_PER_DAY / Self::TICKS_PER_HOUR) as i32;

    /// Cf. TimeSpan.cs#L230
    pub const ZERO: TimeSpan = TimeSpan { ticks: 0 };
    /// Cf. TimeSpan.cs#L232
    pub const MAX: TimeSpan = TimeSpan { ticks: i64::MAX };
    /// Cf. TimeSpan.cs#L233
    pub const MIN: TimeSpan = TimeSpan { ticks: i64::MIN };

    /// Constructs a `TimeSpan` directly from a tick count.
    ///
    /// Also covers the C# static factory `FromTicks(long)`, which is defined as an
    /// alias for this constructor (TimeSpan.cs#L695).
    ///
    /// Cf. TimeSpan.cs#L239-L242
    #[must_use]
    pub const fn from_ticks(ticks: i64) -> Self {
        TimeSpan { ticks }
    }

    /// Cf. TimeSpan.cs#L308
    #[must_use]
    pub const fn ticks(&self) -> i64 {
        self.ticks
    }

    /// Cf. TimeSpan.cs#L310
    #[must_use]
    pub fn days(&self) -> i32 {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "ticks / TICKS_PER_DAY is at most ~10.68 million in magnitude \
                      (i64::MAX / TICKS_PER_DAY), far within i32's range for any representable \
                      TimeSpan"
        )]
        let days = (self.ticks / Self::TICKS_PER_DAY) as i32;
        days
    }

    /// Cf. TimeSpan.cs#L312
    #[must_use]
    pub fn hours(&self) -> i32 {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "modulo by HOURS_PER_DAY (24) bounds the result to (-23..=23), well within i32"
        )]
        let hours = (self.ticks / Self::TICKS_PER_HOUR % i64::from(Self::HOURS_PER_DAY)) as i32;
        hours
    }

    /// Cf. TimeSpan.cs#L334
    #[must_use]
    pub fn minutes(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_MINUTE % Self::MINUTES_PER_HOUR) as i32
    }

    /// Cf. TimeSpan.cs#L336
    #[must_use]
    pub fn seconds(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_SECOND % Self::SECONDS_PER_MINUTE) as i32
    }

    /// Cf. TimeSpan.cs#L314
    #[must_use]
    pub fn milliseconds(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_MILLISECOND % Self::MILLISECONDS_PER_SECOND) as i32
    }

    /// Cf. TimeSpan.cs#L316-L323
    #[must_use]
    pub fn microseconds(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_MICROSECOND % Self::MICROSECONDS_PER_MILLISECOND) as i32
    }

    /// Cf. TimeSpan.cs#L325-L332
    #[must_use]
    pub fn nanoseconds(&self) -> i32 {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "ticks % TICKS_PER_MICROSECOND (10) is in -9..=9, times NANOSECONDS_PER_TICK \
                      (100) is in -900..=900, well within i32"
        )]
        let nanoseconds =
            (self.ticks % Self::TICKS_PER_MICROSECOND * Self::NANOSECONDS_PER_TICK) as i32;
        nanoseconds
    }

    /// Cf. TimeSpan.cs#L338
    #[must_use]
    pub fn total_days(&self) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s TotalDays: `(double)_ticks / TicksPerDay` has the identical \
                      precision-loss characteristics for large tick magnitudes (TimeSpan.cs)"
        )]
        let total_days = self.ticks as f64 / Self::TICKS_PER_DAY as f64;
        total_days
    }

    /// Cf. TimeSpan.cs#L340
    #[must_use]
    pub fn total_hours(&self) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s TotalHours: `(double)_ticks / TicksPerHour` has the identical \
                      precision-loss characteristics for large tick magnitudes (TimeSpan.cs)"
        )]
        let total_hours = self.ticks as f64 / Self::TICKS_PER_HOUR as f64;
        total_hours
    }

    /// Cf. TimeSpan.cs#L385
    #[must_use]
    pub fn total_minutes(&self) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s TotalMinutes: `(double)_ticks / TicksPerMinute` has the identical \
                      precision-loss characteristics for large tick magnitudes (TimeSpan.cs)"
        )]
        let total_minutes = self.ticks as f64 / Self::TICKS_PER_MINUTE as f64;
        total_minutes
    }

    /// Cf. TimeSpan.cs#L387
    #[must_use]
    pub fn total_seconds(&self) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s TotalSeconds: `(double)_ticks / TicksPerSecond` has the identical \
                      precision-loss characteristics for large tick magnitudes (TimeSpan.cs)"
        )]
        let total_seconds = self.ticks as f64 / Self::TICKS_PER_SECOND as f64;
        total_seconds
    }

    /// Clamps to the tick-range boundary (expressed in milliseconds) instead of
    /// overflowing, matching the C# source's explicit clamp rather than raising.
    ///
    /// Cf. TimeSpan.cs#L342-L359
    #[must_use]
    pub fn total_milliseconds(&self) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s MaxMilliseconds constant (long.MaxValue / TicksPerMillisecond) \
                      implicitly converted to double for the comparison/return in TotalMilliseconds \
                      (TimeSpan.cs)"
        )]
        let max = (i64::MAX / Self::TICKS_PER_MILLISECOND) as f64;
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s MinMilliseconds constant, see `max` above"
        )]
        let min = (i64::MIN / Self::TICKS_PER_MILLISECOND) as f64;
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s TotalMilliseconds: `(double)_ticks / TicksPerMillisecond` \
                      (TimeSpan.cs)"
        )]
        let value = self.ticks as f64 / Self::TICKS_PER_MILLISECOND as f64;

        if value > max {
            max
        } else if value < min {
            min
        } else {
            value
        }
    }

    /// Cf. TimeSpan.cs#L361-L371
    #[must_use]
    pub fn total_microseconds(&self) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s TotalMicroseconds: `(double)_ticks / TicksPerMicrosecond` has the \
                      identical precision-loss characteristics for large tick magnitudes (TimeSpan.cs)"
        )]
        let total_microseconds = self.ticks as f64 / Self::TICKS_PER_MICROSECOND as f64;
        total_microseconds
    }

    /// Cf. TimeSpan.cs#L373-L383
    #[must_use]
    pub fn total_nanoseconds(&self) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s TotalNanoseconds: `(double)_ticks * NanosecondsPerTick` has the \
                      identical precision-loss characteristics for large tick magnitudes (TimeSpan.cs)"
        )]
        let total_nanoseconds = self.ticks as f64 * Self::NANOSECONDS_PER_TICK as f64;
        total_nanoseconds
    }

    // --- Everything below is unimplemented: real signatures, `todo!()` bodies. ---
    // Each becomes its own scoped work-issue once drift-scan/work-issue starts
    // iterating on this crate.

    /// Returns a fresh, all-zero [`TimeSpanBuilder`] — a fluent, Rust-idiomatic
    /// multi-component constructor unifying the day/hour/minute/second/millisecond/
    /// microsecond component space around `i64` fields.
    ///
    /// ```
    /// use cs_timespan_automated_v1::TimeSpan;
    ///
    /// let ts = TimeSpan::builder()
    ///     .days(1)
    ///     .hours(2)
    ///     .minutes(30)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(ts, TimeSpan::from_ticks(954_000_000_000));
    /// ```
    #[must_use]
    pub fn builder() -> TimeSpanBuilder {
        TimeSpanBuilder::default()
    }

    /// Performs real tick addition, only reporting [`TimeSpanError::Overflow`] when
    /// the two's-complement sign-bit check used by C#'s `operator+` detects genuine
    /// overflow (identical operand signs, opposite result sign) — e.g.
    /// `TimeSpan::MAX.checked_add(TimeSpan::from_ticks(1))` errors, but
    /// `TimeSpan::MAX.checked_add(TimeSpan::MIN)` correctly returns `-1` tick.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if the sum overflows the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L389 (instance `Add`), TimeSpan.cs#L893-L905 (`operator+`)
    pub fn checked_add(self, rhs: Self) -> Result<Self, TimeSpanError> {
        let result = self.ticks.wrapping_add(rhs.ticks);
        let t1_sign = self.ticks >> 63;
        let t2_sign = rhs.ticks >> 63;
        let result_sign = result >> 63;

        if (t1_sign == t2_sign) && (t1_sign != result_sign) {
            return Err(TimeSpanError::Overflow);
        }
        Ok(TimeSpan { ticks: result })
    }

    /// Performs real tick subtraction, only reporting [`TimeSpanError::Overflow`]
    /// when the two's-complement sign-bit check used by C#'s `operator-` detects
    /// genuine overflow (different operand signs, result sign opposite the
    /// minuend's).
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if the difference overflows the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L687 (instance `Subtract`), TimeSpan.cs#L877-L889 (`operator-`)
    pub fn checked_sub(self, rhs: Self) -> Result<Self, TimeSpanError> {
        let result = self.ticks.wrapping_sub(rhs.ticks);
        let t1_sign = self.ticks >> 63;
        let t2_sign = rhs.ticks >> 63;
        let result_sign = result >> 63;

        if (t1_sign != t2_sign) && (t1_sign != result_sign) {
            return Err(TimeSpanError::Overflow);
        }
        Ok(TimeSpan { ticks: result })
    }

    /// `TimeSpan.MinValue` can't be negated: `-i64::MIN` overflows `i64`. C#'s
    /// `Negate()` delegates to `operator-`, which throws `OverflowException` in
    /// exactly that case.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if `self` is [`TimeSpan::MIN`], whose
    /// magnitude doesn't fit in `i64`.
    ///
    /// Cf. TimeSpan.cs#L683 (instance `Negate`), TimeSpan.cs#L868-L875 (`operator-`)
    pub fn checked_neg(self) -> Result<Self, TimeSpanError> {
        self.ticks
            .checked_neg()
            .map(|ticks| TimeSpan { ticks })
            .ok_or(TimeSpanError::Overflow)
    }

    /// `TimeSpan.MinValue.Duration()` throws `OverflowException` in C#, because
    /// taking the absolute value of `long.MinValue` overflows `i64`.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if `self` is [`TimeSpan::MIN`], whose
    /// magnitude doesn't fit in `i64`.
    ///
    /// Cf. TimeSpan.cs#L416-L423
    pub fn duration(self) -> Result<Self, TimeSpanError> {
        self.ticks
            .checked_abs()
            .map(|ticks| TimeSpan { ticks })
            .ok_or(TimeSpanError::Overflow)
    }

    /// Rounding to the nearest tick is as close to the result we'd have with
    /// unlimited precision as possible, and so likely to have the least potential
    /// to surprise — matching the comment directly above C#'s own `Math.Round` call.
    /// `Math.Round(double)` with no `MidpointRounding` argument rounds half to even
    /// (banker's rounding, e.g. `Math.Round(2.5) == 2.0`), not `f64::round()`'s
    /// round-half-away-from-zero (`2.5f64.round() == 3.0`) — `f64::round_ties_even`
    /// is the match.
    ///
    /// Known, deliberate quirk (not a bug): `checked_neg()` and `checked_mul(-1.0)`
    /// disagree at the `MIN`/`MAX` boundary — e.g. `TimeSpan::MAX.checked_neg()` is
    /// `Ok(-9223372036854775807)` but `TimeSpan::MAX.checked_mul(-1.0)` is
    /// `Ok(TimeSpan::MIN)`, and at `TimeSpan::MIN` they diverge even more sharply
    /// (`Err(Overflow)` vs `Ok(TimeSpan::MAX)`). This isn't a Rust-port bug:
    /// `Negate()`/unary `operator-` in upstream `TimeSpan.cs` does exact `long`
    /// arithmetic (a `ticks == MinTicks` check, no floating point), while
    /// `operator*(TimeSpan, double)` goes through `Math.Round` and
    /// `IntervalFromDoubleTicks`, comparing against `MaxTicks`/`MinTicks` as
    /// implicitly-converted `double`s — and `long.MaxValue as double` rounds up to
    /// `2^63`, not representable exactly, which is where the divergence comes from.
    /// Two genuinely different overflow-detection code paths in C#, faithfully
    /// preserved here — don't "fix" `checked_mul` toward `checked_neg`'s exact-integer
    /// boundary if a property test (re)discovers this.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::NotANumber`] if `factor` is NaN, or
    /// [`TimeSpanError::Overflow`] if the product overflows the range representable
    /// by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L689 (instance `Multiply`), TimeSpan.cs#L907-L919 (`operator *`)
    pub fn checked_mul(self, factor: f64) -> Result<Self, TimeSpanError> {
        if factor.is_nan() {
            return Err(TimeSpanError::NotANumber);
        }

        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s operator*: `timeSpan.Ticks * factor` implicitly promotes Ticks \
                      (long) to double for the multiplication (TimeSpan.cs)"
        )]
        let ticks = (self.ticks as f64 * factor).round_ties_even();
        Self::interval_from_double_ticks(ticks)
    }

    /// Same round-half-to-even rationale as [`Self::checked_mul`] — see its doc
    /// comment.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::NotANumber`] if `divisor` is NaN, or
    /// [`TimeSpanError::Overflow`] if the quotient overflows the range representable
    /// by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L691 (instance `Divide(double)`), TimeSpan.cs#L925-L934
    /// (`operator /`)
    pub fn checked_div(self, divisor: f64) -> Result<Self, TimeSpanError> {
        if divisor.is_nan() {
            return Err(TimeSpanError::NotANumber);
        }

        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s operator/: `timeSpan.Ticks / divisor` implicitly promotes Ticks \
                      (long) to double for the division (TimeSpan.cs)"
        )]
        let ticks = (self.ticks as f64 / divisor).round_ties_even();
        Self::interval_from_double_ticks(ticks)
    }

    /// Infallible: mirrors C#'s floating-point `TimeSpan / TimeSpan`, which can
    /// legitimately produce `f64::INFINITY`/`NAN` rather than erroring.
    ///
    /// Cf. TimeSpan.cs#L693 (instance `Divide(TimeSpan)`), TimeSpan.cs#L936-L941
    #[must_use]
    pub fn divide_time_span(self, rhs: Self) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s `TimeSpan / TimeSpan` operator: `t1.Ticks / (double)t2.Ticks` \
                      implicitly promotes both operands to double (TimeSpan.cs)"
        )]
        let result = self.ticks as f64 / rhs.ticks as f64;
        result
    }

    /// Validates `value` isn't NaN, then scales it into a tick count via
    /// [`Self::interval_from_double_ticks`]. Shared by all six `f64`-argument
    /// `from_*` factories below.
    ///
    /// Cf. TimeSpan.cs#L636-L643 (private `Interval`)
    fn interval(value: f64, scale: f64) -> Result<Self, TimeSpanError> {
        if value.is_nan() {
            return Err(TimeSpanError::NotANumber);
        }
        Self::interval_from_double_ticks(value * scale)
    }

    /// Bounds-checks a tick count already computed as `f64` and converts it to a
    /// `TimeSpan`. `MaxTicks` (`i64::MAX`) isn't exactly representable as `f64` —
    /// it rounds up to `2^63` — so `ticks == MaxTicks` (i.e. `ticks == 2^63` after
    /// that rounding) is special-cased to return [`Self::MAX`] directly rather than
    /// truncating a double that's actually one past the representable range.
    /// `MinTicks` (`i64::MIN`, `-2^63`) needs no such special case: it's an exact
    /// power of two and converts losslessly.
    ///
    /// Cf. TimeSpan.cs#L645-L656 (private `IntervalFromDoubleTicks`)
    #[allow(
        clippy::float_cmp,
        reason = "exact comparison against the well-known MaxTicks rounding boundary, \
                  matching upstream's own `ticks == MaxTicks` check byte-for-byte \
                  (TimeSpan.cs#L649-652) — not an approximate/computed value where an \
                  epsilon comparison would be appropriate"
    )]
    fn interval_from_double_ticks(ticks: f64) -> Result<Self, TimeSpanError> {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s MaxTicks constant (long.MaxValue) implicitly converted to double \
                      for this same bounds comparison (TimeSpan.cs)"
        )]
        let max_ticks = i64::MAX as f64;
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s MinTicks constant, see `max_ticks` above"
        )]
        let min_ticks = i64::MIN as f64;

        if ticks > max_ticks || ticks < min_ticks || ticks.is_nan() {
            return Err(TimeSpanError::Overflow);
        }
        if ticks == max_ticks {
            return Ok(Self::MAX);
        }
        #[allow(
            clippy::cast_possible_truncation,
            reason = "matches C#'s `(long)ticks` truncating cast in IntervalFromDoubleTicks, guarded \
                      by the same bounds check above (TimeSpan.cs)"
        )]
        let ticks = ticks as i64;
        Ok(TimeSpan { ticks })
    }

    /// # Errors
    ///
    /// Returns [`TimeSpanError::NotANumber`] if `value` is NaN, or
    /// [`TimeSpanError::Overflow`] if `value` scaled to ticks falls outside the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L414, TimeSpan.cs#L455
    pub fn from_days(value: f64) -> Result<Self, TimeSpanError> {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s FromDays: `Interval(value, TicksPerDay)` implicitly converts the \
                      TicksPerDay long constant to the double `scale` parameter (TimeSpan.cs)"
        )]
        let scale = Self::TICKS_PER_DAY as f64;
        Self::interval(value, scale)
    }

    /// Single-argument integer overload, bounds-checked against the whole-day
    /// range — distinct from [`Self::from_days`]'s `f64`/`Interval`-based
    /// overload. Named `_i32` (rather than reusing `from_days`) because Rust doesn't
    /// support overloading by parameter type, unlike C#'s `FromDays(int)`/
    /// `FromDays(double)` pair.
    ///
    /// Delegates to [`Self::builder`]: empirically verified (at `MAX_DAYS`,
    /// `MAX_DAYS + 1`, `MIN_DAYS`, `MIN_DAYS - 1`, and interior values) to agree
    /// exactly with the direct bounds-check this used before delegating.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if `days` falls outside the whole-day range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L455
    pub fn from_days_i32(days: i32) -> Result<Self, TimeSpanError> {
        Self::builder().days(i64::from(days)).build()
    }

    /// # Errors
    ///
    /// Returns [`TimeSpanError::NotANumber`] if `value` is NaN, or
    /// [`TimeSpanError::Overflow`] if `value` scaled to ticks falls outside the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L492, TimeSpan.cs#L634
    pub fn from_hours(value: f64) -> Result<Self, TimeSpanError> {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s FromHours: `Interval(value, TicksPerHour)` implicitly converts the \
                      TicksPerHour long constant to the double `scale` parameter (TimeSpan.cs)"
        )]
        let scale = Self::TICKS_PER_HOUR as f64;
        Self::interval(value, scale)
    }

    /// Single-argument integer overload, bounds-checked against the whole-hour
    /// range — distinct from [`Self::from_hours`]'s `f64`/`Interval`-based
    /// overload.
    ///
    /// Delegates to [`Self::builder`]: empirically verified (at `MAX_HOURS`,
    /// `MAX_HOURS + 1`, `MIN_HOURS`, `MIN_HOURS - 1`, and interior values) to
    /// agree exactly with the direct bounds-check this used before delegating.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if `hours` falls outside the whole-hour
    /// range representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L492
    pub fn from_hours_i32(hours: i32) -> Result<Self, TimeSpanError> {
        Self::builder().hours(i64::from(hours)).build()
    }

    /// # Errors
    ///
    /// Returns [`TimeSpanError::NotANumber`] if `value` is NaN, or
    /// [`TimeSpanError::Overflow`] if `value` scaled to ticks falls outside the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L527, TimeSpan.cs#L681
    pub fn from_minutes(value: f64) -> Result<Self, TimeSpanError> {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s FromMinutes: `Interval(value, TicksPerMinute)` implicitly converts \
                      the TicksPerMinute long constant to the double `scale` parameter (TimeSpan.cs)"
        )]
        let scale = Self::TICKS_PER_MINUTE as f64;
        Self::interval(value, scale)
    }

    /// Single-argument integer overload, bounds-checked against the
    /// whole-minute range — distinct from [`Self::from_minutes`]'s
    /// `f64`/`Interval`-based overload. Takes `i64` (rather than `i32`) matching
    /// C#'s `FromMinutes(long)`.
    ///
    /// Delegates to [`Self::builder`]: empirically verified (at `MAX_MINUTES`,
    /// `MAX_MINUTES + 1`, `MIN_MINUTES`, `MIN_MINUTES - 1`, and interior values)
    /// to agree exactly with the direct bounds-check this used before
    /// delegating.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if `minutes` falls outside the
    /// whole-minute range representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L527
    pub fn from_minutes_i64(minutes: i64) -> Result<Self, TimeSpanError> {
        Self::builder().minutes(minutes).build()
    }

    /// # Errors
    ///
    /// Returns [`TimeSpanError::NotANumber`] if `value` is NaN, or
    /// [`TimeSpanError::Overflow`] if `value` scaled to ticks falls outside the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L560, TimeSpan.cs#L685
    pub fn from_seconds(value: f64) -> Result<Self, TimeSpanError> {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s FromSeconds: `Interval(value, TicksPerSecond)` implicitly converts \
                      the TicksPerSecond long constant to the double `scale` parameter (TimeSpan.cs)"
        )]
        let scale = Self::TICKS_PER_SECOND as f64;
        Self::interval(value, scale)
    }

    /// Single-argument integer overload, bounds-checked against the
    /// whole-second range — distinct from [`Self::from_seconds`]'s
    /// `f64`/`Interval`-based overload. Takes `i64` (rather than `i32`) matching
    /// C#'s `FromSeconds(long)`.
    ///
    /// Delegates to [`Self::builder`]: empirically verified (at `MAX_SECONDS`,
    /// `MAX_SECONDS + 1`, `MIN_SECONDS`, `MIN_SECONDS - 1`, and interior values)
    /// to agree exactly with the direct bounds-check this used before
    /// delegating.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if `seconds` falls outside the
    /// whole-second range representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L560
    pub fn from_seconds_i64(seconds: i64) -> Result<Self, TimeSpanError> {
        Self::builder().seconds(seconds).build()
    }

    /// # Errors
    ///
    /// Returns [`TimeSpanError::NotANumber`] if `value` is NaN, or
    /// [`TimeSpanError::Overflow`] if `value` scaled to ticks falls outside the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L591-L592, TimeSpan.cs#L658
    pub fn from_milliseconds(value: f64) -> Result<Self, TimeSpanError> {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s FromMilliseconds: `Interval(value, TicksPerMillisecond)` \
                      implicitly converts the TicksPerMillisecond long constant to the double `scale` \
                      parameter (TimeSpan.cs)"
        )]
        let scale = Self::TICKS_PER_MILLISECOND as f64;
        Self::interval(value, scale)
    }

    /// Single-argument integer overload, bounds-checked against the
    /// whole-millisecond range — distinct from [`Self::from_milliseconds`]'s
    /// `f64`/`Interval`-based overload. Takes `i64` (rather than `i32`) matching
    /// C#'s `FromMilliseconds(long)`.
    ///
    /// Delegates to [`Self::builder`]: empirically verified (at
    /// `MAX_MILLISECONDS`, `MAX_MILLISECONDS + 1`, `MIN_MILLISECONDS`,
    /// `MIN_MILLISECONDS - 1`, and interior values) to agree exactly with the
    /// direct bounds-check this used before delegating.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if `milliseconds` falls outside the
    /// whole-millisecond range representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L591-L592
    pub fn from_milliseconds_i64(milliseconds: i64) -> Result<Self, TimeSpanError> {
        Self::builder().milliseconds(milliseconds).build()
    }

    /// # Errors
    ///
    /// Returns [`TimeSpanError::NotANumber`] if `value` is NaN, or
    /// [`TimeSpanError::Overflow`] if `value` scaled to ticks falls outside the range
    /// representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L632, TimeSpan.cs#L679
    pub fn from_microseconds(value: f64) -> Result<Self, TimeSpanError> {
        #[allow(
            clippy::cast_precision_loss,
            reason = "matches C#'s FromMicroseconds: `Interval(value, TicksPerMicrosecond)` \
                      implicitly converts the TicksPerMicrosecond long constant to the double `scale` \
                      parameter (TimeSpan.cs)"
        )]
        let scale = Self::TICKS_PER_MICROSECOND as f64;
        Self::interval(value, scale)
    }

    /// Single-argument integer overload, bounds-checked against the
    /// whole-microsecond range — distinct from [`Self::from_microseconds`]'s
    /// `f64`/`Interval`-based overload. Takes `i64` (rather than `i32`) matching
    /// C#'s `FromMicroseconds(long)`.
    ///
    /// Delegates to [`Self::builder`]: empirically verified (at
    /// `MAX_MICROSECONDS`, `MAX_MICROSECONDS + 1`, `MIN_MICROSECONDS`,
    /// `MIN_MICROSECONDS - 1`, and interior values) to agree exactly with the
    /// direct bounds-check this used before delegating.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::Overflow`] if `microseconds` falls outside the
    /// whole-microsecond range representable by `TimeSpan`.
    ///
    /// Cf. TimeSpan.cs#L632
    pub fn from_microseconds_i64(microseconds: i64) -> Result<Self, TimeSpanError> {
        Self::builder().microseconds(microseconds).build()
    }

    /// Formats `self` using the given standard or custom format string, mirroring C#'s
    /// `ToString(string? format)` for invariant-culture formats.
    ///
    /// Supports the same single-character standard specifiers as C#'s
    /// `TimeSpanFormat.Format`: an empty string, `"c"`, `"t"`, and `"T"` all produce the
    /// same output as [`Display`](std::fmt::Display) (the constant `"c"` format); `"g"`
    /// produces the general short format (variable-width hours, day segment omitted
    /// when zero, fraction shown only when non-zero with trailing zeros trimmed); `"G"`
    /// produces the general long format (always two-digit hours, day segment always
    /// present, fraction always shown at full 7-digit width). Any other single
    /// character is invalid: C#'s `Format`/`TryFormat` dispatch format strings of
    /// length 1 through this same special case, *before* ever reaching the custom
    /// tokenizer below — so a length-1 format string can never be interpreted as a
    /// custom-format token (e.g. `"d"` alone is invalid, even though `"dd"` is a valid
    /// custom day token).
    ///
    /// Any format string of length != 1 is run through the custom-format-string
    /// mini-language (`TimeSpanFormat.FormatCustomized`): `%d`/`dd`...`dddddddd` (day,
    /// 1-8 digits), `%h`/`hh` (hour), `%m`/`mm` (minute), `%s`/`ss` (second), `%f`/
    /// `ff`...`fffffff` (fraction, truncated to N digits, always shown), `%F`/`FF`...
    /// `FFFFFFF` (fraction, trailing zeros dropped, omitted entirely if the trimmed
    /// value is empty), literal text via `\`-escaping and `'...'`/`"..."` quoting, and
    /// `%` as an escape-next-char marker equivalent to a 1-length token. Unlike the
    /// standard formats above, a custom format string never writes a sign character —
    /// there's no token for it — so a negative `TimeSpan` formats identically to its
    /// positive magnitude (a real upstream quirk, not a bug this port introduces).
    ///
    /// A syntactically invalid custom format string (unterminated quote, a `d`/`f`/`F`
    /// run past its maximum length, an `h`/`m`/`s` run longer than 2, trailing `%` or
    /// `\`, `"%%"`, or an unquoted/unescaped literal character) returns
    /// [`TimeSpanError::InvalidFormat`] rather than panicking, mirroring C#'s
    /// `FormatException`.
    ///
    /// `IFormatProvider`/culture handling is out of scope: like `"c"`, `"g"`/`"G"`'s
    /// decimal separator is hardcoded to `.` (the invariant-culture value) rather than
    /// varying by culture, matching this crate having no culture/locale support
    /// anywhere else. Custom format strings have no built-in fraction-separator token
    /// (callers spell out `"."` themselves, e.g. `"dd\\.ss"`), so this doesn't apply to
    /// the custom-format branch in the same way.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::InvalidFormat`] if `format` is a single character that
    /// isn't one of the five standard specifiers, or a syntactically invalid custom
    /// format string.
    ///
    /// Cf. TimeSpanFormat.cs#L19-L48 (`Format`), TimeSpanFormat.cs#L91-L100 (`FormatG`),
    /// TimeSpanFormat.cs#L109-L294 (`TryFormatStandard`), TimeSpanFormat.cs#L296-455
    /// (`FormatCustomized`)
    ///
    /// ```
    /// use cs_timespan_automated_v1::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().hours(1).minutes(2).seconds(3).build().unwrap();
    /// assert_eq!(ts.to_string_format("c").unwrap(), ts.to_string());
    /// assert_eq!(ts.to_string_format("g").unwrap(), "1:02:03");
    /// assert_eq!(ts.to_string_format("G").unwrap(), "0:01:02:03.0000000");
    /// assert_eq!(ts.to_string_format("hh\\:mm\\:ss").unwrap(), "01:02:03");
    /// ```
    pub fn to_string_format(&self, format: &str) -> Result<String, TimeSpanError> {
        let mut chars = format.chars();
        let first = chars.next();
        let second = chars.next();

        match (first, second) {
            // Empty string and "c"/"t"/"T" are all the same constant format — see this
            // function's doc comment above.
            (None | Some('c' | 't' | 'T'), None) => Ok(self.to_string()),
            (Some('g'), None) => Ok(self.format_general(false)),
            (Some('G'), None) => Ok(self.format_general(true)),
            (Some(_), None) => Err(TimeSpanError::InvalidFormat),
            _ => crate::time_span_format_custom::format_customized(*self, format),
        }
    }

    /// Shared implementation for the general short (`"g"`) and general long (`"G"`)
    /// standard formats. `long` selects `"G"`'s always-two-digit-hours/always-present-
    /// day/always-full-fraction behavior over `"g"`'s variable-width/trimmed behavior.
    ///
    /// Same `i128`-widening rationale as the [`Display`](std::fmt::Display) impl for
    /// handling `TimeSpan::MIN` without overflow.
    ///
    /// Cf. TimeSpanFormat.cs#L109-L294 (`TryFormatStandard`, `StandardFormat.g`/`.G`
    /// branches)
    fn format_general(self, long: bool) -> String {
        use std::fmt::Write;

        let negative = self.ticks < 0;
        let abs_ticks: i128 = if negative {
            -i128::from(self.ticks)
        } else {
            i128::from(self.ticks)
        };

        let ticks_per_second = i128::from(Self::TICKS_PER_SECOND);
        let fraction = Self::fraction_from_abs_ticks(abs_ticks, ticks_per_second);
        let total_seconds = abs_ticks / ticks_per_second;

        let (total_minutes, seconds) = (total_seconds / 60, total_seconds % 60);
        let (total_hours, minutes) = (total_minutes / 60, total_minutes % 60);
        let (days, hours) = (total_hours / 24, total_hours % 24);

        let mut out = String::new();
        if negative {
            out.push('-');
        }

        if days > 0 {
            let _ = write!(out, "{days}:");
        } else if long {
            out.push_str("0:");
        }

        if !long && hours < 10 {
            out.push_str(&hours.to_string());
        } else {
            let _ = write!(out, "{hours:02}");
        }
        let _ = write!(out, ":{minutes:02}:{seconds:02}");

        if long {
            let _ = write!(out, ".{fraction:07}");
        } else if fraction != 0 {
            let (value, digits) = Self::trim_fraction_trailing_zeros(fraction);
            let _ = write!(out, ".{value:0width$}", width = digits as usize);
        }

        out
    }

    /// Extracts the sub-second tick-fraction component from a non-negative tick
    /// magnitude already widened to `i128` (`abs_ticks`, `ticks_per_second` is always
    /// [`Self::TICKS_PER_SECOND`] widened the same way). Shared by
    /// [`Self::format_general`], [`Self::try_format_standard`], and the
    /// [`Display`](std::fmt::Display) impl, which each compute this identically.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "abs_ticks % ticks_per_second is bounded to [0, TICKS_PER_SECOND) since abs_ticks \
                  is always non-negative, well within u32 and never negative"
    )]
    fn fraction_from_abs_ticks(abs_ticks: i128, ticks_per_second: i128) -> u32 {
        (abs_ticks % ticks_per_second) as u32
    }

    /// Renders `hours` as the single ASCII decimal digit `"g"`'s single-digit-hour
    /// case needs, for use by [`Self::try_format_standard`]. Callers pass `hours`
    /// only when it's already known to be `< 10`; combined with `hours` being
    /// `total_hours % 24` (non-negative, `abs_ticks`-derived), it's bounded to
    /// `[0, 9]`, well within `u8`.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "hours is total_hours % 24 (non-negative abs_ticks-derived) and callers only pass \
                  values < 10, bounded to [0, 9], well within u8 and never negative"
    )]
    fn single_hour_digit(hours: i128) -> u8 {
        b'0' + hours as u8
    }

    /// Trims trailing zeros from a 7-digit tick fraction (`0..=9_999_999`), returning
    /// the trimmed value and how many digits it should be zero-padded to when
    /// printed — matching `"g"`'s "write out only the most significant digits"
    /// behavior. Only called when `fraction != 0`.
    ///
    /// Cf. TimeSpanFormat.cs#L166-L174 (`StandardFormat.g` branch's
    /// `FormattingHelpers.CountDecimalTrailingZeros` call)
    fn trim_fraction_trailing_zeros(fraction: u32) -> (u32, u32) {
        debug_assert!(fraction != 0 && fraction < 10_000_000);
        let mut value = fraction;
        let mut digits = 7u32;
        while value.is_multiple_of(10) {
            value /= 10;
            digits -= 1;
        }
        (value, digits)
    }

    /// Parses `input` against a caller-supplied format string, honoring `styles`.
    ///
    /// Mirrors `TimeSpan.ParseExact(string, string, IFormatProvider?, TimeSpanStyles)` /
    /// `TimeSpan.TryParseExact(string, string, IFormatProvider?, TimeSpanStyles, out
    /// TimeSpan)` — Rust's `Result` already distinguishes success from failure the way
    /// those `Try`-prefixed bool-return/`out`-param pairs do, so (matching `FromStr::
    /// from_str`'s precedent for `Parse`/`TryParse`) this single method covers both; there's
    /// no separate infallible variant that panics.
    ///
    /// `format` may be one of C#'s five single-letter standard formats, or a custom format
    /// string:
    ///
    /// - `"c"`, `"t"`, `"T"`: the legacy constant format (`[-]d[.]hh:mm[:ss[.fffffff]]`,
    ///   day segment optional) — all three are the exact same algorithm upstream.
    /// - `"g"`: the general short format (variable-width hours, day segment omitted when
    ///   zero) — the same shape [`TimeSpan::to_string_format`]'s `"g"` produces.
    /// - `"G"`: the general long format (always two-digit hours, day segment always
    ///   present, fraction always present) — the same shape `to_string_format`'s `"G"`
    ///   produces; unlike the other four, `"G"` only accepts that one full shape.
    /// - Any other string: the custom-format-string mini-language — `h`/`hh`, `m`/`mm`,
    ///   `s`/`ss` (1-or-2-digit / exactly-2-digit), `d` through `dddddddd` (1-to-8-digit /
    ///   exactly-N-digit, up to 8), `f` through `fffffff` (exactly N digits required) and
    ///   `F` through `FFFFFFF` (up to N digits, all optional) repeat patterns,
    ///   `'...'`/`"..."` quoted literals, `\`-escaped literal characters, and a leading `%`
    ///   marking a single custom specifier character.
    ///
    /// `styles` is honored only for the custom-format-string case: C#'s dispatch never
    /// passes it through to the standard-format algorithms
    /// (TimeSpanParse.cs#L1237-1241), so `AssumeNegative` has no effect on `"c"`/`"t"`/
    /// `"T"`/`"g"`/`"G"`.
    ///
    /// See [`TimeSpan::parse_exact_multiple`] for the multi-format-string-array overload set
    /// (`ParseExactMultiple`/`TryParseExactMultiple`) — tries each format in a slice in turn,
    /// reusing this method's per-format logic.
    ///
    /// Deferred, and left for follow-up work:
    ///
    /// - Any `IFormatProvider`/culture handling — this crate has none, anywhere (matching
    ///   `FromStr::from_str`'s invariant-culture-only scope).
    ///
    /// See `time_span_parse_exact.rs` for the top-level dispatch and the custom-format-string
    /// algorithm (ported from `TimeSpanParse.cs`'s `TryParseByFormat`),
    /// `time_span_parse_constant.rs` for `"c"`/`"t"`/`"T"` (ported from
    /// `TryParseTimeSpanConstant`/`StringParser`), and `time_span_parse.rs`'s
    /// `parse_general` for `"g"`/`"G"` (ported from `TryParseTimeSpan` with the `Localized`/
    /// `RequireFull` styles).
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::InvalidFormat`] if `format` is empty, a single
    /// character that isn't one of the five standard specifiers, or a syntactically
    /// invalid custom format string, or if `input` doesn't match `format`. Returns
    /// [`TimeSpanError::Overflow`] if `input` matches `format` but the resulting value
    /// falls outside the range representable by `TimeSpan`.
    ///
    /// Cf. TimeSpanParse.cs's `TryParseExactTimeSpan`/`TryParseByFormat`
    /// (TimeSpanParse.cs#L1228-L1416)
    ///
    /// ```
    /// use cs_timespan_automated_v1::{TimeSpan, TimeSpanStyles};
    ///
    /// let ts = TimeSpan::parse_exact("12.23:32:43", r"dd\.h\:m\:s", TimeSpanStyles::None)
    ///     .unwrap();
    /// assert_eq!(
    ///     ts,
    ///     TimeSpan::builder()
    ///         .days(12)
    ///         .hours(23)
    ///         .minutes(32)
    ///         .seconds(43)
    ///         .build()
    ///         .unwrap()
    /// );
    ///
    /// let ts = TimeSpan::parse_exact("1.12:24:02", "c", TimeSpanStyles::None).unwrap();
    /// assert_eq!(
    ///     ts,
    ///     TimeSpan::builder()
    ///         .days(1)
    ///         .hours(12)
    ///         .minutes(24)
    ///         .seconds(2)
    ///         .build()
    ///         .unwrap()
    /// );
    /// ```
    pub fn parse_exact(
        input: &str,
        format: &str,
        styles: TimeSpanStyles,
    ) -> Result<Self, TimeSpanError> {
        crate::time_span_parse_exact::parse_exact(input, format, styles)
    }

    /// Tries each format string in `formats`, in order, against `input`, returning the
    /// result of the first one that matches. Mirrors C#'s `ParseExact(string, string[],
    /// IFormatProvider?)` / `ParseExact(string, string[], IFormatProvider?, TimeSpanStyles)`
    /// / `TryParseExact(string, string[], IFormatProvider?, out TimeSpan)` /
    /// `TryParseExact(string, string[], IFormatProvider?, TimeSpanStyles, out TimeSpan)` —
    /// collapsed into one `Result`-returning method for the same reason
    /// [`TimeSpan::parse_exact`] is (see its doc comment).
    ///
    /// Each format string may be any of the forms [`TimeSpan::parse_exact`] accepts (the
    /// five single-letter standard formats or the custom-format-string mini-language);
    /// `styles` applies to every attempt exactly as it does for a single `parse_exact` call.
    ///
    /// Notable edge cases, matching upstream:
    ///
    /// - Empty `input` is *unconditionally* a bad-format failure
    ///   ([`TimeSpanError::InvalidFormat`]), checked before `formats` is even inspected —
    ///   unlike [`TimeSpan::parse_exact`], which has no such check and will happily parse
    ///   `""` against a format that matches empty input (e.g. a custom format consisting
    ///   solely of an empty quoted literal, `"''"`).
    /// - An empty `formats` slice is itself a bad-format failure
    ///   ([`TimeSpanError::InvalidFormat`]), distinct from any individual format being bad.
    ///   (C#'s `formats == null` case has no `&str` equivalent here — a `&[&str]` can't be
    ///   null.)
    /// - An empty individual format string anywhere in `formats` is a bad-format-specifier
    ///   failure *immediately* — the loop stops right there rather than skipping that entry
    ///   in favor of a later one that might otherwise have matched.
    /// - Each attempt is independent: a failure on format `N` — including an overflow that
    ///   would be reported as [`TimeSpanError::Overflow`] from a standalone
    ///   [`TimeSpan::parse_exact`] call — never leaks into the attempt on format `N+1`, and
    ///   is never itself returned; if every format in the slice fails, the result is always
    ///   the generic [`TimeSpanError::InvalidFormat`], regardless of why any individual
    ///   attempt failed.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::InvalidFormat`] if `input` is empty, `formats` is
    /// empty, any individual format string in `formats` is empty, or `input` doesn't
    /// match any format in `formats` — see the notable-edge-cases list above for how
    /// each of those is distinguished from a per-attempt [`TimeSpanError::Overflow`],
    /// which is never itself returned from this method.
    ///
    /// Cf. `TimeSpanParse.cs`'s `TryParseExactMultipleTimeSpan`
    /// (TimeSpanParse.cs#L1662-1703)
    ///
    /// ```
    /// use cs_timespan_automated_v1::{TimeSpan, TimeSpanStyles};
    ///
    /// // "hh\:mm\:ss" doesn't match "3" (no digits/colon shape to match), so the array
    /// // falls through to "%h", which does.
    /// let ts = TimeSpan::parse_exact_multiple(
    ///     "3",
    ///     &[r"hh\:mm\:ss", "%h"],
    ///     TimeSpanStyles::None,
    /// )
    /// .unwrap();
    /// assert_eq!(ts, TimeSpan::builder().hours(3).build().unwrap());
    /// ```
    pub fn parse_exact_multiple(
        input: &str,
        formats: &[&str],
        styles: TimeSpanStyles,
    ) -> Result<Self, TimeSpanError> {
        crate::time_span_parse_exact::parse_exact_multiple(input, formats, styles)
    }

    /// Formats `self` into `destination` using the given standard format string,
    /// writing UTF-8 bytes directly rather than allocating a `String` — the
    /// non-allocating counterpart to [`TimeSpan::to_string_format`]. Mirrors C#'s
    /// `bool TryFormat(Span<char> destination, out int charsWritten, ...)` and its
    /// `IUtf8SpanFormattable` `bool TryFormat(Span<byte> utf8Destination, out int
    /// bytesWritten, ...)` overload.
    ///
    /// C# has two `TryFormat` overloads because `Span<char>` (UTF-16 code units) and
    /// `Span<byte>` (UTF-8 code units) differ in encoding width for non-ASCII text.
    /// Every standard-format output this crate produces (digits, `-`, `.`, `:`) is
    /// ASCII, so a single `u8` buffer serves both roles identically: it's
    /// simultaneously valid UTF-8 and a one-byte-per-character buffer. A caller
    /// wanting a `&str` view can pass it through `std::str::from_utf8` infallibly,
    /// since the output is always ASCII — so no separate char-buffer overload is
    /// needed here, unlike C#'s two generic `TChar` instantiations.
    ///
    /// Supports the same format strings as [`TimeSpan::to_string_format`]: an empty
    /// string, the standard `"c"`/`"t"`/`"T"`/`"g"`/`"G"` single-character formats, and
    /// the custom-format-string mini-language (`%d`/`dd`...`dddddddd`, `%h`/`hh`, `%m`/
    /// `mm`, `%s`/`ss`, `%f`/`ff`...`fffffff`, `%F`/`FF`...`FFFFFFF`, `'...'`/`"..."`
    /// quoting, `\`-escaping — see `to_string_format`'s doc comment for the full
    /// rundown). A single character that isn't one of the five standard formats, or a
    /// syntactically invalid custom format string, returns
    /// [`TimeSpanError::InvalidFormat`], checked before `destination`'s length (so an
    /// invalid format string is reported even when `destination` is too short to hold
    /// any output).
    ///
    /// The custom-format path is not zero-allocation: it formats into an intermediate
    /// `String` via the same [`format_customized`](crate::time_span_format_custom::format_customized)
    /// helper `to_string_format` uses, then copies those bytes into `destination` (or
    /// reports [`TimeSpanError::InsufficientBuffer`] without copying anything, if it
    /// doesn't fit) — unlike the standard-format path, which computes the required
    /// length and writes `destination` directly with no intermediate allocation. This
    /// mirrors upstream's own shape: C#'s `TryFormat<TChar>` builds custom-format
    /// output into a scratch `ValueListBuilder<TChar>` (stack-allocated up to 256
    /// `TChar`s, spilling to the heap beyond that) before copying it into the caller's
    /// `destination` via `ValueListBuilder.TryCopyTo` — which is itself all-or-nothing,
    /// writing nothing and returning `false` if `destination` is too small. A
    /// hand-written buffer-writing tokenizer that avoided the intermediate allocation
    /// entirely would need to duplicate `format_customized`'s digit/fraction-writing
    /// logic against a `&mut [u8]` target instead of `String` — a larger, riskier
    /// rewrite for a colder path than the standard formats; this crate accepts the one
    /// intermediate allocation instead.
    ///
    /// Returns the number of bytes written on success. Returns
    /// [`TimeSpanError::InsufficientBuffer`] — writing nothing — when `destination` is
    /// shorter than the formatted output requires, mirroring `TryFormatStandard`'s
    /// `false`-with-`charsWritten = 0` behavior rather than panicking or writing a
    /// truncated prefix. A `destination` exactly long enough succeeds and is filled
    /// completely; a larger `destination` succeeds and leaves any trailing bytes past
    /// the written prefix untouched.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanError::InvalidFormat`] if `format` is a single character that
    /// isn't one of the five standard specifiers, or a syntactically invalid custom
    /// format string — checked before `destination`'s length, per this doc comment's
    /// note above. Returns [`TimeSpanError::InsufficientBuffer`] if `format` is valid
    /// but `destination` is too short to hold the formatted output.
    ///
    /// Cf. TimeSpanFormat.cs#L50-L82 (`TryFormat<TChar>`), TimeSpanFormat.cs#L109-L294
    /// (`TryFormatStandard<TChar>`, its `requiredOutputLength` computation and
    /// insufficient-space `false` return), TimeSpanFormat.cs#L77-81 (`TryFormat`'s
    /// custom-format branch: scratch `ValueListBuilder<TChar>` then `TryCopyTo`),
    /// ValueListBuilder.cs#L149-159 (`TryCopyTo`'s all-or-nothing contract)
    ///
    /// ```
    /// use cs_timespan_automated_v1::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().hours(1).minutes(2).seconds(3).build().unwrap();
    /// let mut buf = [0u8; 32];
    /// let written = ts.try_format(&mut buf, "c").unwrap();
    /// assert_eq!(&buf[..written], b"01:02:03");
    ///
    /// let written = ts.try_format(&mut buf, "hh\\:mm\\:ss").unwrap();
    /// assert_eq!(&buf[..written], b"01:02:03");
    /// ```
    pub fn try_format(&self, destination: &mut [u8], format: &str) -> Result<usize, TimeSpanError> {
        let mut chars = format.chars();
        let first = chars.next();
        let second = chars.next();

        let standard = match (first, second) {
            // Empty string and "c"/"t"/"T" are all the same constant format — see
            // `to_string_format`'s doc comment.
            (None | Some('c' | 't' | 'T'), None) => StandardFormat::Constant,
            (Some('g'), None) => StandardFormat::GeneralShort,
            (Some('G'), None) => StandardFormat::GeneralLong,
            (Some(_), None) => return Err(TimeSpanError::InvalidFormat),
            _ => return self.try_format_customized(destination, format),
        };

        self.try_format_standard(standard, destination)
    }

    /// Non-allocating* custom-format-string counterpart to [`TimeSpan::try_format`]'s
    /// standard-format path, backing `try_format` for any format string that isn't one
    /// of the five standard specifiers. See `try_format`'s doc comment for the
    /// allocation tradeoff this makes (one intermediate `String`, mirroring C#'s own
    /// scratch-buffer-then-copy shape).
    ///
    /// Cf. TimeSpanFormat.cs#L77-81 (`TryFormat`'s custom-format branch),
    /// ValueListBuilder.cs#L149-159 (`TryCopyTo`)
    fn try_format_customized(
        self,
        destination: &mut [u8],
        format: &str,
    ) -> Result<usize, TimeSpanError> {
        let formatted = crate::time_span_format_custom::format_customized(self, format)?;
        let bytes = formatted.as_bytes();

        if destination.len() < bytes.len() {
            return Err(TimeSpanError::InsufficientBuffer);
        }

        destination[..bytes.len()].copy_from_slice(bytes);
        Ok(bytes.len())
    }

    /// Shared implementation backing [`TimeSpan::try_format`] for all three standard
    /// formats. Computes the exact required output length up front (mirroring
    /// `TryFormatStandard`'s `requiredOutputLength` computation field-by-field) before
    /// writing a single byte, so an undersized `destination` is rejected without any
    /// partial write — matching C#'s all-or-nothing `false` return.
    ///
    /// Cf. TimeSpanFormat.cs#L109-L294 (`TryFormatStandard<TChar>`)
    fn try_format_standard(
        self,
        format: StandardFormat,
        destination: &mut [u8],
    ) -> Result<usize, TimeSpanError> {
        let negative = self.ticks < 0;
        let abs_ticks: i128 = if negative {
            -i128::from(self.ticks)
        } else {
            i128::from(self.ticks)
        };

        let ticks_per_second = i128::from(Self::TICKS_PER_SECOND);
        let mut fraction = Self::fraction_from_abs_ticks(abs_ticks, ticks_per_second);
        let total_seconds = abs_ticks / ticks_per_second;

        let (total_minutes, seconds) = (total_seconds / 60, total_seconds % 60);
        let (total_hours, minutes) = (total_minutes / 60, total_minutes % 60);
        let (days, hours) = (total_hours / 24, total_hours % 24);

        // Start with "hh:mm:ss" and adjust as necessary, mirroring
        // TryFormatStandard's requiredOutputLength computation exactly so the
        // insufficient-space case triggers at the right buffer length.
        let mut required_output_length: usize = 8;
        if negative {
            required_output_length += 1; // leading '-'
        }

        let fraction_digits: u32 = match format {
            StandardFormat::Constant => {
                // "c": a fraction only when non-zero, always all 7 digits.
                if fraction != 0 {
                    required_output_length += 8; // 7 digits + leading '.'
                    7
                } else {
                    0
                }
            }
            StandardFormat::GeneralLong => {
                // "G": a fraction unconditionally, always all 7 digits.
                required_output_length += 8; // 7 digits + 1-char decimal separator
                7
            }
            StandardFormat::GeneralShort => {
                // "g": a fraction only when non-zero, trailing zeros trimmed.
                if fraction != 0 {
                    let (trimmed, digits) = Self::trim_fraction_trailing_zeros(fraction);
                    fraction = trimmed;
                    required_output_length += digits as usize + 1; // digits + separator
                    digits
                } else {
                    0
                }
            }
        };

        let mut hour_digits: usize = 2;
        if format == StandardFormat::GeneralShort && hours < 10 {
            // "g": a single-digit hour rather than the usual two-digit hour.
            hour_digits = 1;
            required_output_length -= 1;
        }

        let day_digits: usize = if days > 0 {
            let digits = Self::count_digits_i128(days);
            required_output_length += digits + 1; // digits + leading "d." or "d:"
            digits
        } else if format == StandardFormat::GeneralLong {
            // "G": a leading "0:" even when days is 0.
            required_output_length += 2;
            1
        } else {
            0
        };

        if destination.len() < required_output_length {
            return Err(TimeSpanError::InsufficientBuffer);
        }

        let mut pos = 0;
        if negative {
            destination[pos] = b'-';
            pos += 1;
        }

        if day_digits != 0 {
            pos = Self::write_padded_digits(destination, pos, days, day_digits);
            destination[pos] = if format == StandardFormat::Constant {
                b'.'
            } else {
                b':'
            };
            pos += 1;
        }

        if hour_digits == 2 {
            pos = Self::write_padded_digits(destination, pos, hours, 2);
        } else {
            destination[pos] = Self::single_hour_digit(hours);
            pos += 1;
        }
        destination[pos] = b':';
        pos += 1;
        pos = Self::write_padded_digits(destination, pos, minutes, 2);
        destination[pos] = b':';
        pos += 1;
        pos = Self::write_padded_digits(destination, pos, seconds, 2);

        if fraction_digits != 0 {
            destination[pos] = b'.';
            pos += 1;
            pos = Self::write_padded_digits(
                destination,
                pos,
                i128::from(fraction),
                fraction_digits as usize,
            );
        }

        debug_assert_eq!(pos, required_output_length);
        Ok(pos)
    }

    /// Writes `value` into `destination` at `pos` as exactly `width` ASCII decimal
    /// digits, zero-padded on the left, and returns the position just past what was
    /// written. `value` must fit within `width` digits — callers compute `width` from
    /// `value` itself (via [`TimeSpan::count_digits_i128`]) or from a fixed format
    /// width known to be large enough (e.g. `2` for minutes/seconds).
    fn write_padded_digits(destination: &mut [u8], pos: usize, value: i128, width: usize) -> usize {
        debug_assert!(value >= 0);
        let mut remaining = value;
        for i in (0..width).rev() {
            #[allow(
                clippy::cast_sign_loss,
                reason = "remaining starts non-negative (see debug_assert! above) and dividing/taking \
                          the modulo of a non-negative i128 by the positive constant 10 keeps it \
                          non-negative, so `remaining % 10` is in [0, 9] — never negative"
            )]
            let digit = b'0' + (remaining % 10) as u8;
            destination[pos + i] = digit;
            remaining /= 10;
        }
        debug_assert_eq!(remaining, 0, "value did not fit within `width` digits");
        pos + width
    }

    /// Number of decimal digits needed to write `value` (`value >= 0`) without
    /// leading zeros. Mirrors `FormattingHelpers.CountDigits` as used by
    /// `TryFormatStandard`'s day-segment length calculation.
    fn count_digits_i128(value: i128) -> usize {
        debug_assert!(value >= 0);
        if value == 0 {
            return 1;
        }
        let mut remaining = value;
        let mut digits = 0;
        while remaining > 0 {
            digits += 1;
            remaining /= 10;
        }
        digits
    }
}

/// The standard format specifiers [`TimeSpan::try_format`] supports, mirroring
/// `TimeSpanFormat`'s internal `StandardFormat` enum (`C`/`G`/`g`). `"t"`/`"T"` and an
/// empty format string all map to [`StandardFormat::Constant`], same as
/// [`TimeSpan::to_string_format`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StandardFormat {
    /// The constant `"c"` format (and its `"t"`/`"T"`/empty-string aliases).
    Constant,
    /// The general short `"g"` format.
    GeneralShort,
    /// The general long `"G"` format.
    GeneralLong,
}

/// Built on [`TimeSpan::checked_neg`]. Rust's `Neg` trait can't return a `Result`,
/// so overflow panics here, mirroring C#'s `OverflowException` from `operator-`.
///
/// Cf. TimeSpan.cs#L868-L875
impl std::ops::Neg for TimeSpan {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.checked_neg()
            .expect("TimeSpan negation overflowed its representable range")
    }
}

/// Built on [`TimeSpan::checked_sub`]. Rust's `Sub` trait can't return a `Result`,
/// so overflow panics here, mirroring C#'s `OverflowException` from `operator-`.
///
/// Cf. TimeSpan.cs#L877-L889
impl std::ops::Sub for TimeSpan {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
            .expect("TimeSpan subtraction overflowed its representable range")
    }
}

/// Built on [`TimeSpan::checked_add`]. Rust's `Add` trait can't return a `Result`,
/// so overflow panics here, mirroring C#'s `OverflowException` from `operator+`.
///
/// Cf. TimeSpan.cs#L893-L905
impl std::ops::Add for TimeSpan {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs)
            .expect("TimeSpan addition overflowed its representable range")
    }
}

/// Built on [`TimeSpan::checked_mul`]. Rust's `Mul` trait can't return a `Result`,
/// so overflow (or a NaN `factor`) panics here, mirroring C#'s `OverflowException`/
/// `ArgumentException` from `operator*`.
///
/// Cf. TimeSpan.cs#L907-L919
impl std::ops::Mul<f64> for TimeSpan {
    type Output = Self;

    fn mul(self, factor: f64) -> Self::Output {
        self.checked_mul(factor)
            .expect("TimeSpan multiplication overflowed its representable range, or factor was NaN")
    }
}

/// Delegates to `TimeSpan * f64`, mirroring C#'s own `operator *(double, TimeSpan)`,
/// which is itself defined purely as `timeSpan * factor`.
///
/// Cf. TimeSpan.cs#L921-L922
impl std::ops::Mul<TimeSpan> for f64 {
    type Output = TimeSpan;

    fn mul(self, timespan: TimeSpan) -> Self::Output {
        timespan * self
    }
}

/// Built on [`TimeSpan::checked_div`]. Rust's `Div` trait can't return a `Result`,
/// so overflow (or a NaN `divisor`) panics here, mirroring C#'s `OverflowException`/
/// `ArgumentException` from `operator/`.
///
/// Cf. TimeSpan.cs#L925-L934
impl std::ops::Div<f64> for TimeSpan {
    type Output = Self;

    fn div(self, divisor: f64) -> Self::Output {
        self.checked_div(divisor)
            .expect("TimeSpan division overflowed its representable range, or divisor was NaN")
    }
}

/// Cf. TimeSpan.cs#L936-L941
impl std::ops::Div<TimeSpan> for TimeSpan {
    type Output = f64;

    fn div(self, rhs: TimeSpan) -> Self::Output {
        self.divide_time_span(rhs)
    }
}

/// Invariant-culture parsing only, mirroring `Parse(string)`/`TryParse`. See
/// `time_span_parse.rs` for the algorithm (ported from `TimeSpanParse.cs`).
///
/// C#'s `IFormatProvider`-aware overloads (`Parse(string, IFormatProvider?)`,
/// `TryParse(..., IFormatProvider?, ...)`) and `TryFormat` remain deferred — this crate
/// has no culture/locale support anywhere. `ToString(string? format)`'s standard `"g"`/
/// `"G"` formats are covered by [`TimeSpan::to_string_format`]. The custom-format-string
/// `ParseExact`/`TryParseExact` family is now covered (single format-string overload
/// only) by [`TimeSpan::parse_exact`]; see its doc comment for what's still deferred
/// there.
///
/// Cf. TimeSpan.cs#L722-L727
///
/// ```
/// use cs_timespan_automated_v1::TimeSpan;
///
/// let ts: TimeSpan = "1.02:03:04".parse().unwrap();
/// assert_eq!(ts.days(), 1);
/// assert_eq!(ts.hours(), 2);
/// assert_eq!(ts.minutes(), 3);
/// assert_eq!(ts.seconds(), 4);
/// ```
impl FromStr for TimeSpan {
    type Err = TimeSpanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::time_span_parse::parse(s)
    }
}

/// The invariant, culture-independent constant `"c"` format —
/// `[-][d.]hh:mm:ss[.fffffff]`, matching `TimeSpanFormat.FormatC`/`TryFormatStandard`
/// (`StandardFormat.C`); C#'s parameterless `ToString()` delegates to the same format.
/// See [`TimeSpan::to_string_format`] for the `"g"`/`"G"` general-format equivalents of
/// `ToString(string? format)`. Custom format strings (`TimeSpanFormat.FormatCustomized`),
/// char/UTF-8 `TryFormat`, and any `IFormatProvider` handling remain deferred — see the
/// follow-up issues tracking that remainder.
///
/// The days component, when present, is written with no leading-zero padding (C#
/// writes exactly `FormattingHelpers.CountDigits(days)` digits); the fraction, when
/// non-zero, is always written with all 7 digits (ticks are 100ns units, so the
/// fractional-second remainder needs up to 7 digits) — matching `"c"`'s "write out all
/// 7 digits" behavior, as opposed to `"g"`'s trimmed-trailing-zeros behavior.
///
/// `i64::MIN`'s magnitude doesn't fit in `i64` (`-i64::MIN` overflows), so ticks are
/// widened to `i128` before negating, mirroring `TryFormatStandard`'s explicit
/// `long.MinValue` special-case.
///
/// Cf. TimeSpan.cs#L855 (invariant `"c"` format), TimeSpanFormat.cs (`FormatC`,
/// `TryFormatStandard` with `StandardFormat.C`)
impl std::fmt::Display for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let negative = self.ticks < 0;
        let abs_ticks: i128 = if negative {
            -i128::from(self.ticks)
        } else {
            i128::from(self.ticks)
        };

        let ticks_per_second = i128::from(Self::TICKS_PER_SECOND);
        let fraction = Self::fraction_from_abs_ticks(abs_ticks, ticks_per_second);
        let total_seconds = abs_ticks / ticks_per_second;

        let (total_minutes, seconds) = (total_seconds / 60, total_seconds % 60);
        let (total_hours, minutes) = (total_minutes / 60, total_minutes % 60);
        let (days, hours) = (total_hours / 24, total_hours % 24);

        if negative {
            write!(f, "-")?;
        }
        if days > 0 {
            write!(f, "{days}.")?;
        }
        write!(f, "{hours:02}:{minutes:02}:{seconds:02}")?;
        if fraction != 0 {
            write!(f, ".{fraction:07}")?;
        }
        Ok(())
    }
}
