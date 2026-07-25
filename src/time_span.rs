use std::cmp::Ordering;
use std::str::FromStr;

use crate::error::TimeSpanError;

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

    pub const HOURS_PER_DAY: i32 = (Self::TICKS_PER_DAY / Self::TICKS_PER_HOUR) as i32;

    /// Cf. TimeSpan.cs#L230
    pub const ZERO: TimeSpan = TimeSpan { ticks: 0 };
    /// Cf. TimeSpan.cs#L232
    pub const MAX: TimeSpan = TimeSpan { ticks: i64::MAX };
    /// Cf. TimeSpan.cs#L233
    pub const MIN: TimeSpan = TimeSpan { ticks: i64::MIN };

    /// Cf. TimeSpan.cs#L225-L226 (internal `MinDays`/`MaxDays`)
    const MIN_DAYS: i64 = i64::MIN / Self::TICKS_PER_DAY;
    const MAX_DAYS: i64 = i64::MAX / Self::TICKS_PER_DAY;

    /// Cf. TimeSpan.cs#L222-L223 (internal `MinHours`/`MaxHours`)
    const MIN_HOURS: i64 = i64::MIN / Self::TICKS_PER_HOUR;
    const MAX_HOURS: i64 = i64::MAX / Self::TICKS_PER_HOUR;

    /// Cf. TimeSpan.cs#L219-L220 (internal `MinMinutes`/`MaxMinutes`)
    const MIN_MINUTES: i64 = i64::MIN / Self::TICKS_PER_MINUTE;
    const MAX_MINUTES: i64 = i64::MAX / Self::TICKS_PER_MINUTE;

    /// Cf. TimeSpan.cs#L216-L217 (internal `MinSeconds`/`MaxSeconds`)
    const MIN_SECONDS: i64 = i64::MIN / Self::TICKS_PER_SECOND;
    const MAX_SECONDS: i64 = i64::MAX / Self::TICKS_PER_SECOND;

    /// Cf. TimeSpan.cs#L213-L214 (internal `MinMilliseconds`/`MaxMilliseconds`)
    const MIN_MILLISECONDS: i64 = i64::MIN / Self::TICKS_PER_MILLISECOND;
    const MAX_MILLISECONDS: i64 = i64::MAX / Self::TICKS_PER_MILLISECOND;

    /// Cf. TimeSpan.cs#L210-L211 (internal `MinMicroseconds`/`MaxMicroseconds`)
    const MIN_MICROSECONDS: i64 = i64::MIN / Self::TICKS_PER_MICROSECOND;
    const MAX_MICROSECONDS: i64 = i64::MAX / Self::TICKS_PER_MICROSECOND;

    /// Sums hours/minutes/seconds into a validated tick count. Widens to `i128`
    /// while summing so the addition itself can never overflow (unlike the C#
    /// source, whose comment notes `totalSeconds` is bounded well within 64 bits
    /// for realistic inputs); the out-of-range check below is what actually
    /// enforces the representable range, matching `ArgumentOutOfRangeException`.
    ///
    /// Cf. TimeSpan.cs#L698-L711 (internal `TimeToTicks`)
    fn time_to_ticks(hours: i32, minutes: i32, seconds: i32) -> Result<i64, TimeSpanError> {
        let total_seconds = (hours as i128) * (Self::SECONDS_PER_HOUR as i128)
            + (minutes as i128) * (Self::SECONDS_PER_MINUTE as i128)
            + seconds as i128;

        if total_seconds > Self::MAX_SECONDS as i128 || total_seconds < Self::MIN_SECONDS as i128 {
            return Err(TimeSpanError::Overflow);
        }

        Ok(total_seconds as i64 * Self::TICKS_PER_SECOND)
    }

    /// Sums days/hours/minutes/seconds/milliseconds/microseconds into a
    /// validated tick count. Same `i128` widening rationale as [`Self::time_to_ticks`].
    ///
    /// Cf. TimeSpan.cs#L292-L306 (6-arg constructor body)
    fn dhms_to_ticks(
        days: i32,
        hours: i32,
        minutes: i32,
        seconds: i32,
        milliseconds: i32,
        microseconds: i32,
    ) -> Result<i64, TimeSpanError> {
        let total_microseconds = (days as i128) * (Self::MICROSECONDS_PER_DAY as i128)
            + (hours as i128) * (Self::MICROSECONDS_PER_HOUR as i128)
            + (minutes as i128) * (Self::MICROSECONDS_PER_MINUTE as i128)
            + (seconds as i128) * (Self::MICROSECONDS_PER_SECOND as i128)
            + (milliseconds as i128) * (Self::MICROSECONDS_PER_MILLISECOND as i128)
            + microseconds as i128;

        if total_microseconds > Self::MAX_MICROSECONDS as i128
            || total_microseconds < Self::MIN_MICROSECONDS as i128
        {
            return Err(TimeSpanError::Overflow);
        }

        Ok(total_microseconds as i64 * Self::TICKS_PER_MICROSECOND)
    }

    /// Constructs a `TimeSpan` directly from a tick count.
    ///
    /// Also covers the C# static factory `FromTicks(long)`, which is defined as an
    /// alias for this constructor (TimeSpan.cs#L695).
    ///
    /// Cf. TimeSpan.cs#L239-L242
    pub const fn from_ticks(ticks: i64) -> Self {
        TimeSpan { ticks }
    }

    /// Cf. TimeSpan.cs#L308
    pub const fn ticks(&self) -> i64 {
        self.ticks
    }

    /// Cf. TimeSpan.cs#L310
    pub fn days(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_DAY) as i32
    }

    /// Cf. TimeSpan.cs#L312
    pub fn hours(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_HOUR % Self::HOURS_PER_DAY as i64) as i32
    }

    /// Cf. TimeSpan.cs#L334
    pub fn minutes(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_MINUTE % Self::MINUTES_PER_HOUR) as i32
    }

    /// Cf. TimeSpan.cs#L336
    pub fn seconds(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_SECOND % Self::SECONDS_PER_MINUTE) as i32
    }

    /// Cf. TimeSpan.cs#L314
    pub fn milliseconds(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_MILLISECOND % Self::MILLISECONDS_PER_SECOND) as i32
    }

    /// Cf. TimeSpan.cs#L316-L323
    pub fn microseconds(&self) -> i32 {
        (self.ticks / Self::TICKS_PER_MICROSECOND % Self::MICROSECONDS_PER_MILLISECOND) as i32
    }

    /// Cf. TimeSpan.cs#L325-L332
    pub fn nanoseconds(&self) -> i32 {
        (self.ticks % Self::TICKS_PER_MICROSECOND * Self::NANOSECONDS_PER_TICK) as i32
    }

    /// Cf. TimeSpan.cs#L338
    pub fn total_days(&self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_DAY as f64
    }

    /// Cf. TimeSpan.cs#L340
    pub fn total_hours(&self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_HOUR as f64
    }

    /// Cf. TimeSpan.cs#L385
    pub fn total_minutes(&self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_MINUTE as f64
    }

    /// Cf. TimeSpan.cs#L387
    pub fn total_seconds(&self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_SECOND as f64
    }

    /// Clamps to the tick-range boundary (expressed in milliseconds) instead of
    /// overflowing, matching the C# source's explicit clamp rather than raising.
    ///
    /// Cf. TimeSpan.cs#L342-L359
    pub fn total_milliseconds(&self) -> f64 {
        let max = (i64::MAX / Self::TICKS_PER_MILLISECOND) as f64;
        let min = (i64::MIN / Self::TICKS_PER_MILLISECOND) as f64;
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
    pub fn total_microseconds(&self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_MICROSECOND as f64
    }

    /// Cf. TimeSpan.cs#L373-L383
    pub fn total_nanoseconds(&self) -> f64 {
        self.ticks as f64 * Self::NANOSECONDS_PER_TICK as f64
    }

    // --- Everything below is unimplemented: real signatures, `todo!()` bodies. ---
    // Each becomes its own scoped work-issue once drift-scan/work-issue starts
    // iterating on this crate.

    /// Cf. TimeSpan.cs#L244-L247
    pub fn from_hms(hours: i32, minutes: i32, seconds: i32) -> Result<Self, TimeSpanError> {
        Ok(Self {
            ticks: Self::time_to_ticks(hours, minutes, seconds)?,
        })
    }

    /// Cf. TimeSpan.cs#L249-L252
    pub fn from_dhms(
        days: i32,
        hours: i32,
        minutes: i32,
        seconds: i32,
    ) -> Result<Self, TimeSpanError> {
        Self::from_dhms_milli(days, hours, minutes, seconds, 0)
    }

    /// Cf. TimeSpan.cs#L254-L273
    pub fn from_dhms_milli(
        days: i32,
        hours: i32,
        minutes: i32,
        seconds: i32,
        milliseconds: i32,
    ) -> Result<Self, TimeSpanError> {
        Self::from_dhms_micro(days, hours, minutes, seconds, milliseconds, 0)
    }

    /// Cf. TimeSpan.cs#L275-L306
    pub fn from_dhms_micro(
        days: i32,
        hours: i32,
        minutes: i32,
        seconds: i32,
        milliseconds: i32,
        microseconds: i32,
    ) -> Result<Self, TimeSpanError> {
        Ok(Self {
            ticks: Self::dhms_to_ticks(days, hours, minutes, seconds, milliseconds, microseconds)?,
        })
    }

    /// Cf. TimeSpan.cs#L394 (`static Compare`)
    pub fn compare(t1: Self, t2: Self) -> Ordering {
        t1.cmp(&t2)
    }

    /// Cf. TimeSpan.cs#L429 (`static Equals`)
    pub fn equals(t1: Self, t2: Self) -> bool {
        t1 == t2
    }

    /// Performs real tick addition, only reporting [`TimeSpanError::Overflow`] when
    /// the two's-complement sign-bit check used by C#'s `operator+` detects genuine
    /// overflow (identical operand signs, opposite result sign) — e.g.
    /// `TimeSpan::MAX.checked_add(TimeSpan::from_ticks(1))` errors, but
    /// `TimeSpan::MAX.checked_add(TimeSpan::MIN)` correctly returns `-1` tick.
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
    /// Cf. TimeSpan.cs#L416-L423
    pub fn duration(self) -> Result<Self, TimeSpanError> {
        self.ticks
            .checked_abs()
            .map(|ticks| TimeSpan { ticks })
            .ok_or(TimeSpanError::Overflow)
    }

    /// Cf. TimeSpan.cs#L689 (instance `Multiply`)
    pub fn checked_mul(self, _factor: f64) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Cf. TimeSpan.cs#L691 (instance `Divide(double)`)
    pub fn checked_div(self, _divisor: f64) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Infallible: mirrors C#'s floating-point `TimeSpan / TimeSpan`, which can
    /// legitimately produce `f64::INFINITY`/`NAN` rather than erroring.
    ///
    /// Cf. TimeSpan.cs#L693 (instance `Divide(TimeSpan)`), TimeSpan.cs#L936-L941
    pub fn divide_time_span(self, rhs: Self) -> f64 {
        self.ticks as f64 / rhs.ticks as f64
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
    fn interval_from_double_ticks(ticks: f64) -> Result<Self, TimeSpanError> {
        let max_ticks = i64::MAX as f64;
        let min_ticks = i64::MIN as f64;

        if ticks > max_ticks || ticks < min_ticks || ticks.is_nan() {
            return Err(TimeSpanError::Overflow);
        }
        if ticks == max_ticks {
            return Ok(Self::MAX);
        }
        Ok(TimeSpan {
            ticks: ticks as i64,
        })
    }

    /// Bounds-checks a raw unit count against `[min_units, max_units]` before
    /// converting to ticks. Shared by the single-argument integer `FromX`
    /// overloads below (e.g. [`Self::from_days_i32`]) — a distinct validation
    /// path from both the `f64`/`Interval`-based overloads (e.g.
    /// [`Self::from_days`]) and the multi-component `_parts` constructors (e.g.
    /// [`Self::from_days_parts`]).
    ///
    /// `min_units`/`max_units` are always `i64::MIN`/`i64::MAX` divided by
    /// `ticks_per_unit` (truncating division), so `units * ticks_per_unit` can't
    /// overflow once `units` has passed the range check.
    ///
    /// Cf. TimeSpan.cs#L433-L444 (private `FromUnits`)
    fn from_units(
        units: i64,
        ticks_per_unit: i64,
        min_units: i64,
        max_units: i64,
    ) -> Result<Self, TimeSpanError> {
        if units > max_units || units < min_units {
            return Err(TimeSpanError::Overflow);
        }
        Ok(TimeSpan {
            ticks: units * ticks_per_unit,
        })
    }

    /// Cf. TimeSpan.cs#L414, TimeSpan.cs#L455
    pub fn from_days(value: f64) -> Result<Self, TimeSpanError> {
        Self::interval(value, Self::TICKS_PER_DAY as f64)
    }

    /// Single-argument integer overload, bounds-checked against the whole-day
    /// range via [`Self::from_units`] — distinct from [`Self::from_days`]'s
    /// `f64`/`Interval`-based overload and [`Self::from_days_parts`]'s
    /// multi-component constructor. Named `_i32` (rather than reusing
    /// `from_days`) because Rust doesn't support overloading by parameter type,
    /// unlike C#'s `FromDays(int)`/`FromDays(double)` pair.
    ///
    /// Cf. TimeSpan.cs#L455
    pub fn from_days_i32(days: i32) -> Result<Self, TimeSpanError> {
        Self::from_units(
            days as i64,
            Self::TICKS_PER_DAY,
            Self::MIN_DAYS,
            Self::MAX_DAYS,
        )
    }

    /// Cf. TimeSpan.cs#L471-L481. The C# overload takes optional trailing
    /// parameters (`hours = 0, minutes = 0, ...`); Rust has no default arguments,
    /// so all components are required here.
    pub fn from_days_parts(
        _days: i32,
        _hours: i32,
        _minutes: i64,
        _seconds: i64,
        _milliseconds: i64,
        _microseconds: i64,
    ) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Cf. TimeSpan.cs#L492, TimeSpan.cs#L634
    pub fn from_hours(value: f64) -> Result<Self, TimeSpanError> {
        Self::interval(value, Self::TICKS_PER_HOUR as f64)
    }

    /// Single-argument integer overload, bounds-checked against the whole-hour
    /// range via [`Self::from_units`] — distinct from [`Self::from_hours`]'s
    /// `f64`/`Interval`-based overload and [`Self::from_hours_parts`]'s
    /// multi-component constructor.
    ///
    /// Cf. TimeSpan.cs#L492
    pub fn from_hours_i32(hours: i32) -> Result<Self, TimeSpanError> {
        Self::from_units(
            hours as i64,
            Self::TICKS_PER_HOUR,
            Self::MIN_HOURS,
            Self::MAX_HOURS,
        )
    }

    /// Cf. TimeSpan.cs#L507-L516
    pub fn from_hours_parts(
        _hours: i32,
        _minutes: i64,
        _seconds: i64,
        _milliseconds: i64,
        _microseconds: i64,
    ) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Cf. TimeSpan.cs#L527, TimeSpan.cs#L681
    pub fn from_minutes(value: f64) -> Result<Self, TimeSpanError> {
        Self::interval(value, Self::TICKS_PER_MINUTE as f64)
    }

    /// Single-argument integer overload, bounds-checked against the
    /// whole-minute range via [`Self::from_units`] — distinct from
    /// [`Self::from_minutes`]'s `f64`/`Interval`-based overload and
    /// [`Self::from_minutes_parts`]'s multi-component constructor. Takes `i64`
    /// (rather than `i32`) matching C#'s `FromMinutes(long)`.
    ///
    /// Cf. TimeSpan.cs#L527
    pub fn from_minutes_i64(minutes: i64) -> Result<Self, TimeSpanError> {
        Self::from_units(
            minutes,
            Self::TICKS_PER_MINUTE,
            Self::MIN_MINUTES,
            Self::MAX_MINUTES,
        )
    }

    /// Cf. TimeSpan.cs#L541-L549
    pub fn from_minutes_parts(
        _minutes: i64,
        _seconds: i64,
        _milliseconds: i64,
        _microseconds: i64,
    ) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Cf. TimeSpan.cs#L560, TimeSpan.cs#L685
    pub fn from_seconds(value: f64) -> Result<Self, TimeSpanError> {
        Self::interval(value, Self::TICKS_PER_SECOND as f64)
    }

    /// Single-argument integer overload, bounds-checked against the
    /// whole-second range via [`Self::from_units`] — distinct from
    /// [`Self::from_seconds`]'s `f64`/`Interval`-based overload and
    /// [`Self::from_seconds_parts`]'s multi-component constructor. Takes `i64`
    /// (rather than `i32`) matching C#'s `FromSeconds(long)`.
    ///
    /// Cf. TimeSpan.cs#L560
    pub fn from_seconds_i64(seconds: i64) -> Result<Self, TimeSpanError> {
        Self::from_units(
            seconds,
            Self::TICKS_PER_SECOND,
            Self::MIN_SECONDS,
            Self::MAX_SECONDS,
        )
    }

    /// Cf. TimeSpan.cs#L573-L580
    pub fn from_seconds_parts(
        _seconds: i64,
        _milliseconds: i64,
        _microseconds: i64,
    ) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Cf. TimeSpan.cs#L591-L592, TimeSpan.cs#L658
    pub fn from_milliseconds(value: f64) -> Result<Self, TimeSpanError> {
        Self::interval(value, Self::TICKS_PER_MILLISECOND as f64)
    }

    /// Single-argument integer overload, bounds-checked against the
    /// whole-millisecond range via [`Self::from_units`] — distinct from
    /// [`Self::from_milliseconds`]'s `f64`/`Interval`-based overload and
    /// [`Self::from_milliseconds_parts`]'s multi-component constructor. Takes
    /// `i64` (rather than `i32`) matching C#'s `FromMilliseconds(long)`.
    ///
    /// Cf. TimeSpan.cs#L591-L592
    pub fn from_milliseconds_i64(milliseconds: i64) -> Result<Self, TimeSpanError> {
        Self::from_units(
            milliseconds,
            Self::TICKS_PER_MILLISECOND,
            Self::MIN_MILLISECONDS,
            Self::MAX_MILLISECONDS,
        )
    }

    /// Cf. TimeSpan.cs#L604-L610
    pub fn from_milliseconds_parts(
        _milliseconds: i64,
        _microseconds: i64,
    ) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Cf. TimeSpan.cs#L632, TimeSpan.cs#L679
    pub fn from_microseconds(value: f64) -> Result<Self, TimeSpanError> {
        Self::interval(value, Self::TICKS_PER_MICROSECOND as f64)
    }

    /// Formats `self` using the given standard format string, mirroring C#'s
    /// `ToString(string? format)` for the invariant-culture standard formats only.
    ///
    /// Supports the same single-character standard specifiers as C#'s
    /// `TimeSpanFormat.Format`: an empty string, `"c"`, `"t"`, and `"T"` all produce the
    /// same output as [`Display`](std::fmt::Display) (the constant `"c"` format); `"g"`
    /// produces the general short format (variable-width hours, day segment omitted
    /// when zero, fraction shown only when non-zero with trailing zeros trimmed); `"G"`
    /// produces the general long format (always two-digit hours, day segment always
    /// present, fraction always shown at full 7-digit width).
    ///
    /// Any other single character, or any format string of length != 1 (including
    /// otherwise-valid C# custom format strings like `"dd\\.ss"`), returns
    /// [`TimeSpanError::InvalidFormat`] rather than panicking, mirroring C#'s
    /// `FormatException` — this crate doesn't implement `TimeSpanFormat.FormatCustomized`
    /// (the custom-format-string tokenizer) yet. See the follow-up issue tracking that
    /// remainder.
    ///
    /// `IFormatProvider`/culture handling is out of scope: like `"c"`, `"g"`/`"G"`'s
    /// decimal separator is hardcoded to `.` (the invariant-culture value) rather than
    /// varying by culture, matching this crate having no culture/locale support
    /// anywhere else.
    ///
    /// Cf. TimeSpanFormat.cs#L19-L48 (`Format`), TimeSpanFormat.cs#L91-L100 (`FormatG`),
    /// TimeSpanFormat.cs#L109-L294 (`TryFormatStandard`)
    ///
    /// ```
    /// use cs_timespan_automated_v1::TimeSpan;
    ///
    /// let ts = TimeSpan::from_hms(1, 2, 3).unwrap();
    /// assert_eq!(ts.to_string_format("c").unwrap(), ts.to_string());
    /// assert_eq!(ts.to_string_format("g").unwrap(), "1:02:03");
    /// assert_eq!(ts.to_string_format("G").unwrap(), "0:01:02:03.0000000");
    /// ```
    pub fn to_string_format(&self, format: &str) -> Result<String, TimeSpanError> {
        let mut chars = format.chars();
        let first = chars.next();
        let second = chars.next();

        match (first, second) {
            (None, None) => Ok(self.to_string()),
            (Some('c' | 't' | 'T'), None) => Ok(self.to_string()),
            (Some('g'), None) => Ok(self.format_general(false)),
            (Some('G'), None) => Ok(self.format_general(true)),
            _ => Err(TimeSpanError::InvalidFormat),
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
    fn format_general(&self, long: bool) -> String {
        let negative = self.ticks < 0;
        let abs_ticks: i128 = if negative {
            -(self.ticks as i128)
        } else {
            self.ticks as i128
        };

        let ticks_per_second = Self::TICKS_PER_SECOND as i128;
        let fraction = (abs_ticks % ticks_per_second) as u32;
        let total_seconds = abs_ticks / ticks_per_second;

        let (total_minutes, seconds) = (total_seconds / 60, total_seconds % 60);
        let (total_hours, minutes) = (total_minutes / 60, total_minutes % 60);
        let (days, hours) = (total_hours / 24, total_hours % 24);

        let mut out = String::new();
        if negative {
            out.push('-');
        }

        if days > 0 {
            out.push_str(&format!("{days}:"));
        } else if long {
            out.push_str("0:");
        }

        if !long && hours < 10 {
            out.push_str(&hours.to_string());
        } else {
            out.push_str(&format!("{hours:02}"));
        }
        out.push_str(&format!(":{minutes:02}:{seconds:02}"));

        if long {
            out.push_str(&format!(".{fraction:07}"));
        } else if fraction != 0 {
            let (value, digits) = Self::trim_fraction_trailing_zeros(fraction);
            out.push_str(&format!(".{value:0width$}", width = digits as usize));
        }

        out
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

/// Cf. TimeSpan.cs#L907-L919
impl std::ops::Mul<f64> for TimeSpan {
    type Output = Self;

    fn mul(self, _factor: f64) -> Self::Output {
        todo!()
    }
}

/// Cf. TimeSpan.cs#L921-L922
impl std::ops::Mul<TimeSpan> for f64 {
    type Output = TimeSpan;

    fn mul(self, _timespan: TimeSpan) -> Self::Output {
        todo!()
    }
}

/// Cf. TimeSpan.cs#L924-L934
impl std::ops::Div<f64> for TimeSpan {
    type Output = Self;

    fn div(self, _divisor: f64) -> Self::Output {
        todo!()
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
/// C#'s `IFormatProvider`-aware overloads and the custom-format-string
/// `ParseExact`/`TryParseExact`/`TryFormat` family remain deferred: their shape depends
/// on more of `TimeSpanParse.cs`/`TimeSpanFormat.cs` than the invariant standard-format
/// grammar this impl covers, so this doesn't guess at a Rust equivalent for
/// custom-format-string/culture handling. (`ToString(string? format)`'s standard `"g"`/
/// `"G"` formats are covered by [`TimeSpan::to_string_format`]; its custom-format-string
/// branch is not.)
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
            -(self.ticks as i128)
        } else {
            self.ticks as i128
        };

        let ticks_per_second = Self::TICKS_PER_SECOND as i128;
        let fraction = (abs_ticks % ticks_per_second) as u32;
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
