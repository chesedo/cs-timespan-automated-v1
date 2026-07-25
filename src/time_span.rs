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

    /// Cf. TimeSpan.cs#L216-L217 (internal `MinSeconds`/`MaxSeconds`)
    const MIN_SECONDS: i64 = i64::MIN / Self::TICKS_PER_SECOND;
    const MAX_SECONDS: i64 = i64::MAX / Self::TICKS_PER_SECOND;

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

    /// Cf. TimeSpan.cs#L414, TimeSpan.cs#L455
    pub fn from_days(_value: f64) -> Result<Self, TimeSpanError> {
        todo!()
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
    pub fn from_hours(_value: f64) -> Result<Self, TimeSpanError> {
        todo!()
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
    pub fn from_minutes(_value: f64) -> Result<Self, TimeSpanError> {
        todo!()
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
    pub fn from_seconds(_value: f64) -> Result<Self, TimeSpanError> {
        todo!()
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
    pub fn from_milliseconds(_value: f64) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Cf. TimeSpan.cs#L604-L610
    pub fn from_milliseconds_parts(
        _milliseconds: i64,
        _microseconds: i64,
    ) -> Result<Self, TimeSpanError> {
        todo!()
    }

    /// Cf. TimeSpan.cs#L632, TimeSpan.cs#L679
    pub fn from_microseconds(_value: f64) -> Result<Self, TimeSpanError> {
        todo!()
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
/// `ParseExact`/`TryParseExact`/`ToString(format)`/`TryFormat` family remain deferred:
/// their shape depends on more of `TimeSpanParse.cs`/`TimeSpanFormat.cs` than the
/// invariant standard-format grammar this impl covers, so this doesn't guess at a Rust
/// equivalent for custom-format-string/culture handling.
///
/// Cf. TimeSpan.cs#L722-L727
impl FromStr for TimeSpan {
    type Err = TimeSpanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::time_span_parse::parse(s)
    }
}

/// Cf. TimeSpan.cs#L855 (invariant `"c"` format)
impl std::fmt::Display for TimeSpan {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
