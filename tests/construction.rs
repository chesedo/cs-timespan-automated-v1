//! Tests for `TimeSpan` construction: constructors, component accessors,
//! equality/ordering, and the `from_*` factory functions.

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError};

/// Mirrors the C# test helper `VerifyTimeSpan(TimeSpan, int, int, int, int, int)`
/// (TimeSpanTests.cs#L1686-L1695), minus its `Assert.Equal(timeSpan, +timeSpan)`
/// check: C# ports the unary `+` operator as an identity, but Rust has no unary `+`
/// operator to overload, so there is nothing to port for that assertion.
fn verify_time_span(
    ts: TimeSpan,
    days: i32,
    hours: i32,
    minutes: i32,
    seconds: i32,
    milliseconds: i32,
) {
    assert_eq!(days, ts.days());
    assert_eq!(hours, ts.hours());
    assert_eq!(minutes, ts.minutes());
    assert_eq!(seconds, ts.seconds());
    assert_eq!(milliseconds, ts.milliseconds());
}

/// Cf. TimeSpanTests.cs#L15-19
#[test]
fn max_value() {
    verify_time_span(TimeSpan::MAX, 10675199, 2, 48, 5, 477);
}

/// Cf. TimeSpanTests.cs#L21-25
#[test]
fn min_value() {
    verify_time_span(TimeSpan::MIN, -10675199, -2, -48, -5, -477);
}

/// Cf. TimeSpanTests.cs#L27-31
#[test]
fn zero() {
    verify_time_span(TimeSpan::ZERO, 0, 0, 0, 0, 0);
}

/// Cf. TimeSpanTests.cs#L33-38 (`Ctor_Empty`). C# tests both `new TimeSpan()` and
/// `default(TimeSpan)`; Rust's `Default` impl is the equivalent of both.
#[test]
fn ctor_empty() {
    verify_time_span(TimeSpan::default(), 0, 0, 0, 0, 0);
}

/// Cf. TimeSpanTests.cs#L40-44
#[test]
fn ctor_long() {
    verify_time_span(
        TimeSpan::from_ticks(999999999999999999),
        1157407,
        9,
        46,
        39,
        999,
    );
}

/// Cf. TimeSpanTests.cs#L126-145 (`Total_Days_Hours_Minutes_Seconds_Milliseconds`).
/// The C# cases are built via the multi-component constructor, which isn't
/// implemented yet; each case here is rebuilt from an equivalent tick count using
/// the already-real per-unit constants instead.
#[test]
fn total_days_hours_minutes_seconds_milliseconds() {
    let cases: [(i64, f64, f64, f64, f64, f64); 5] = [
        (0, 0.0, 0.0, 0.0, 0.0, 0.0),
        (
            500 * TimeSpan::TICKS_PER_MILLISECOND,
            0.5 / 60.0 / 60.0 / 24.0,
            0.5 / 60.0 / 60.0,
            0.5 / 60.0,
            0.5,
            500.0,
        ),
        (
            TimeSpan::TICKS_PER_HOUR,
            1.0 / 24.0,
            1.0,
            60.0,
            3600.0,
            3_600_000.0,
        ),
        (
            TimeSpan::TICKS_PER_DAY,
            1.0,
            24.0,
            1440.0,
            86400.0,
            86_400_000.0,
        ),
        (
            TimeSpan::TICKS_PER_DAY + TimeSpan::TICKS_PER_HOUR,
            25.0 / 24.0,
            25.0,
            1500.0,
            90000.0,
            90_000_000.0,
        ),
    ];

    for (
        ticks,
        expected_days,
        expected_hours,
        expected_minutes,
        expected_seconds,
        expected_milliseconds,
    ) in cases
    {
        let ts = TimeSpan::from_ticks(ticks);
        assert!((expected_days - ts.total_days()).abs() < 1e-12);
        assert!((expected_hours - ts.total_hours()).abs() < 1e-9);
        assert!((expected_minutes - ts.total_minutes()).abs() < 1e-9);
        assert!((expected_seconds - ts.total_seconds()).abs() < 1e-9);
        assert!((expected_milliseconds - ts.total_milliseconds()).abs() < 1e-6);
    }
}

/// Cf. TimeSpanTests.cs#L147-154 (`TotalMilliseconds_Invalid`; despite the name,
/// this checks clamping at the extremes, not an error path).
#[test]
fn total_milliseconds_extremes() {
    let max_milliseconds = (i64::MAX / TimeSpan::TICKS_PER_MILLISECOND) as f64;
    let min_milliseconds = (i64::MIN / TimeSpan::TICKS_PER_MILLISECOND) as f64;

    assert_eq!(max_milliseconds, TimeSpan::MAX.total_milliseconds());
    assert_eq!(min_milliseconds, TimeSpan::MIN.total_milliseconds());
}

/// Cf. TimeSpanTests.cs#L299-349 (`Equals_TestData`/`EqualsTest`), restricted to the
/// two ticks-only rows (L327-328) that don't need the multi-component constructor.
#[test]
fn equals_via_operators() {
    assert_eq!(TimeSpan::from_ticks(10000), TimeSpan::from_ticks(10000));
    assert_ne!(TimeSpan::from_ticks(10000), TimeSpan::from_ticks(20000));
}

/// Cf. TimeSpan.cs#L429 (`static Equals`): `public static bool Equals(TimeSpan t1,
/// TimeSpan t2) => t1 == t2;` — a plain delegation onto the equality operator.
#[test]
fn static_equals() {
    assert!(TimeSpan::equals(
        TimeSpan::from_ticks(5),
        TimeSpan::from_ticks(5)
    ));
    assert!(!TimeSpan::equals(
        TimeSpan::from_ticks(5),
        TimeSpan::from_ticks(6)
    ));
}

/// Cf. TimeSpan.cs#L394 (`static Compare`): `public static int Compare(TimeSpan t1,
/// TimeSpan t2) => t1._ticks.CompareTo(t2._ticks);` — a plain delegation onto the
/// tick count's total order.
#[test]
fn static_compare() {
    use std::cmp::Ordering;

    assert_eq!(
        Ordering::Less,
        TimeSpan::compare(TimeSpan::from_ticks(1), TimeSpan::from_ticks(2))
    );
    assert_eq!(
        Ordering::Equal,
        TimeSpan::compare(TimeSpan::from_ticks(2), TimeSpan::from_ticks(2))
    );
    assert_eq!(
        Ordering::Greater,
        TimeSpan::compare(TimeSpan::from_ticks(3), TimeSpan::from_ticks(2))
    );
}

/// Mirrors the C# test helper's 6-arg overload, which adds a `Microseconds` check
/// on top of the 5-arg helper.
///
/// Cf. TimeSpanTests.cs#L1697-L1702
fn verify_time_span_micro(
    ts: TimeSpan,
    days: i32,
    hours: i32,
    minutes: i32,
    seconds: i32,
    milliseconds: i32,
    microseconds: i32,
) {
    verify_time_span(ts, days, hours, minutes, seconds, milliseconds);
    assert_eq!(microseconds, ts.microseconds());
}

/// Mirrors the C# test helper's 7-arg overload, which adds a `Nanoseconds` check
/// on top of the 6-arg helper.
///
/// Cf. TimeSpanTests.cs#L1704-L1709
fn verify_time_span_nano(
    ts: TimeSpan,
    days: i32,
    hours: i32,
    minutes: i32,
    seconds: i32,
    milliseconds: i32,
    microseconds: i32,
    nanoseconds: i32,
) {
    verify_time_span_micro(
        ts,
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
    );
    assert_eq!(nanoseconds, ts.nanoseconds());
}

/// Cf. TimeSpanTests.cs#L87-92 (`Ctor_Int_Int_Int_Int_Int_Int`). The C# case is
/// built via the 6-component constructor, which isn't implemented yet; rebuilt here
/// from an equivalent tick count using the already-real per-unit constants.
#[test]
fn ctor_dhms_micro_equivalent() {
    let ticks = 10 * TimeSpan::TICKS_PER_DAY
        + 9 * TimeSpan::TICKS_PER_HOUR
        + 8 * TimeSpan::TICKS_PER_MINUTE
        + 7 * TimeSpan::TICKS_PER_SECOND
        + 6 * TimeSpan::TICKS_PER_MILLISECOND
        + 5 * TimeSpan::TICKS_PER_MICROSECOND;

    verify_time_span_micro(TimeSpan::from_ticks(ticks), 10, 9, 8, 7, 6, 5);
}

/// Cf. TimeSpanTests.cs#L114-124 (`Ctor_Int_Int_Int_Int_Int_Int_WithNanosecond`).
/// Same adaptation as above: builds the base instant from ticks instead of the
/// still-stubbed constructor, then adds the extra ticks a nanosecond remainder
/// would contribute, exactly as the C# test does.
#[test]
fn ctor_dhms_micro_with_nanosecond_equivalent() {
    let base_ticks = 10 * TimeSpan::TICKS_PER_DAY
        + 9 * TimeSpan::TICKS_PER_HOUR
        + 8 * TimeSpan::TICKS_PER_MINUTE
        + 7 * TimeSpan::TICKS_PER_SECOND
        + 6 * TimeSpan::TICKS_PER_MILLISECOND
        + 5 * TimeSpan::TICKS_PER_MICROSECOND;

    for nanoseconds in [100i32, 300, 900] {
        let ts = TimeSpan::from_ticks(base_ticks + (nanoseconds / 100) as i64);
        verify_time_span_nano(ts, 10, 9, 8, 7, 6, 5, nanoseconds);
    }
}

/// Cf. TimeSpanTests.cs#L1909-L1917 (`TestTotalMicroseconds`)
#[test]
fn total_microseconds() {
    let cases: [(i64, f64); 3] = [(0, 0.0), (100, 10.0), (1_000, 100.0)];

    for (ticks, expected) in cases {
        assert_eq!(expected, TimeSpan::from_ticks(ticks).total_microseconds());
    }
}

/// Cf. TimeSpanTests.cs#L1919-L1927 (`TestTotalNanoseconds`)
#[test]
fn total_nanoseconds() {
    let cases: [(i64, f64); 3] = [(0, 0.0), (100, 10_000.0), (1_000, 100_000.0)];

    for (ticks, expected) in cases {
        assert_eq!(expected, TimeSpan::from_ticks(ticks).total_nanoseconds());
    }
}

/// Cf. TimeSpanTests.cs#L46-L51 (`Ctor_Int_Int_Int`)
#[test]
fn ctor_hms() {
    let time_span = TimeSpan::from_hms(10, 9, 8).unwrap();
    verify_time_span(time_span, 0, 10, 9, 8, 0);
}

/// Cf. TimeSpanTests.cs#L53-L58 (`Ctor_Int_Int_Int_Invalid`)
#[test]
fn ctor_hms_invalid() {
    assert!(TimeSpan::from_hms(TimeSpan::MIN.total_hours() as i32 - 1, 0, 0).is_err());
    assert!(TimeSpan::from_hms(TimeSpan::MAX.total_hours() as i32 + 1, 0, 0).is_err());
}

/// Sanity coverage for the 4-arg constructor, which C# defines purely as a
/// delegation to the 5-arg overload with `milliseconds = 0` (TimeSpan.cs#L249-L252)
/// and has no dedicated upstream test of its own.
#[test]
fn ctor_dhms() {
    let time_span = TimeSpan::from_dhms(10, 9, 8, 7).unwrap();
    verify_time_span(time_span, 10, 9, 8, 7, 0);
}

/// Cf. TimeSpanTests.cs#L60-L65 (`Ctor_Int_Int_Int_Int_Int`)
#[test]
fn ctor_dhms_milli() {
    let time_span = TimeSpan::from_dhms_milli(10, 9, 8, 7, 6).unwrap();
    verify_time_span(time_span, 10, 9, 8, 7, 6);
}

/// Cf. TimeSpanTests.cs#L67-L85 (`Ctor_Int_Int_Int_Int_Int_Invalid`)
#[test]
fn ctor_dhms_milli_invalid() {
    let min = TimeSpan::MIN;
    assert!(
        TimeSpan::from_dhms_milli(
            min.days() - 1,
            min.hours(),
            min.minutes(),
            min.seconds(),
            min.milliseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_milli(
            min.days(),
            min.hours() - 1,
            min.minutes(),
            min.seconds(),
            min.milliseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_milli(
            min.days(),
            min.hours(),
            min.minutes() - 1,
            min.seconds(),
            min.milliseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_milli(
            min.days(),
            min.hours(),
            min.minutes(),
            min.seconds() - 1,
            min.milliseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_milli(
            min.days(),
            min.hours(),
            min.minutes(),
            min.seconds(),
            min.milliseconds() - 1
        )
        .is_err()
    );

    let max = TimeSpan::MAX;
    assert!(
        TimeSpan::from_dhms_milli(
            max.days() + 1,
            max.hours(),
            max.minutes(),
            max.seconds(),
            max.milliseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_milli(
            max.days(),
            max.hours() + 1,
            max.minutes(),
            max.seconds(),
            max.milliseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_milli(
            max.days(),
            max.hours(),
            max.minutes() + 1,
            max.seconds(),
            max.milliseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_milli(
            max.days(),
            max.hours(),
            max.minutes(),
            max.seconds() + 1,
            max.milliseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_milli(
            max.days(),
            max.hours(),
            max.minutes(),
            max.seconds(),
            max.milliseconds() + 1
        )
        .is_err()
    );
}

/// Cf. TimeSpanTests.cs#L87-L92 (`Ctor_Int_Int_Int_Int_Int_Int`)
#[test]
fn ctor_dhms_micro() {
    let time_span = TimeSpan::from_dhms_micro(10, 9, 8, 7, 6, 5).unwrap();
    verify_time_span_micro(time_span, 10, 9, 8, 7, 6, 5);
}

/// Cf. TimeSpanTests.cs#L94-L112 (`Ctor_Int_Int_Int_Int_Int_Int_Invalid`)
#[test]
fn ctor_dhms_micro_invalid() {
    let min = TimeSpan::MIN;
    assert!(
        TimeSpan::from_dhms_micro(
            min.days() - 1,
            min.hours(),
            min.minutes(),
            min.seconds(),
            min.milliseconds(),
            min.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            min.days(),
            min.hours() - 1,
            min.minutes(),
            min.seconds(),
            min.milliseconds(),
            min.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            min.days(),
            min.hours(),
            min.minutes() - 1,
            min.seconds(),
            min.milliseconds(),
            min.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            min.days(),
            min.hours(),
            min.minutes(),
            min.seconds() - 1,
            min.milliseconds(),
            min.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            min.days(),
            min.hours(),
            min.minutes(),
            min.seconds(),
            min.milliseconds() - 1,
            min.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            min.days(),
            min.hours(),
            min.minutes(),
            min.seconds(),
            min.milliseconds(),
            min.microseconds() - 1
        )
        .is_err()
    );

    let max = TimeSpan::MAX;
    assert!(
        TimeSpan::from_dhms_micro(
            max.days() + 1,
            max.hours(),
            max.minutes(),
            max.seconds(),
            max.milliseconds(),
            max.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            max.days(),
            max.hours() + 1,
            max.minutes(),
            max.seconds(),
            max.milliseconds(),
            max.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            max.days(),
            max.hours(),
            max.minutes() + 1,
            max.seconds(),
            max.milliseconds(),
            max.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            max.days(),
            max.hours(),
            max.minutes(),
            max.seconds() + 1,
            max.milliseconds(),
            max.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            max.days(),
            max.hours(),
            max.minutes(),
            max.seconds(),
            max.milliseconds() + 1,
            max.microseconds()
        )
        .is_err()
    );
    assert!(
        TimeSpan::from_dhms_micro(
            max.days(),
            max.hours(),
            max.minutes(),
            max.seconds(),
            max.milliseconds(),
            max.microseconds() + 1
        )
        .is_err()
    );
}

/// The single-argument integer `FromDays`/etc. overloads are bounds-checked against
/// that unit's whole `Min*`/`Max*` constant via the private `FromUnits` helper —
/// distinct from both the `f64`/`Interval`-based overload (`TimeSpan::from_days`) and
/// the multi-component `_parts` constructor (`TimeSpan::from_days_parts`).
///
/// Cf. TimeSpan.cs#L455, TimeSpanTests.cs#L507-L516 (`FromDays_Int_Single_ShouldCreate`)
#[test]
fn from_days_i32_basic() {
    assert_eq!(TimeSpan::from_hms(0, 0, 0), TimeSpan::from_days_i32(0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_DAY)),
        TimeSpan::from_days_i32(1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_DAY)),
        TimeSpan::from_days_i32(-1)
    );

    const MAX_DAYS: i32 = 10_675_199;
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            MAX_DAYS as i64 * TimeSpan::TICKS_PER_DAY
        )),
        TimeSpan::from_days_i32(MAX_DAYS)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -MAX_DAYS as i64 * TimeSpan::TICKS_PER_DAY
        )),
        TimeSpan::from_days_i32(-MAX_DAYS)
    );
}

/// Cf. TimeSpan.cs#L433-L444 (`FromUnits`), TimeSpanTests.cs#L518-L524
/// (`FromDays_Int_Single_ShouldOverflow`)
#[test]
fn from_days_i32_overflow() {
    const MAX_DAYS: i32 = 10_675_199;
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_i32(MAX_DAYS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_i32(-(MAX_DAYS + 1))
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_i32(i32::MAX)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_i32(i32::MIN)
    );
}

/// Cf. TimeSpan.cs#L492, TimeSpanTests.cs#L532-L539
/// (`FromHours_Int_Single_ShouldCreate`)
#[test]
fn from_hours_i32_basic() {
    assert_eq!(TimeSpan::from_hms(0, 0, 0), TimeSpan::from_hours_i32(0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR)),
        TimeSpan::from_hours_i32(1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_HOUR)),
        TimeSpan::from_hours_i32(-1)
    );

    const MAX_HOURS: i32 = 256_204_778;
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            MAX_HOURS as i64 * TimeSpan::TICKS_PER_HOUR
        )),
        TimeSpan::from_hours_i32(MAX_HOURS)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -MAX_HOURS as i64 * TimeSpan::TICKS_PER_HOUR
        )),
        TimeSpan::from_hours_i32(-MAX_HOURS)
    );
}

/// Cf. TimeSpan.cs#L433-L444 (`FromUnits`), TimeSpanTests.cs#L541-L547
/// (`FromHours_Int_Single_ShouldOverflow`)
#[test]
fn from_hours_i32_overflow() {
    const MAX_HOURS: i32 = 256_204_778;
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_i32(MAX_HOURS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_i32(-(MAX_HOURS + 1))
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_i32(i32::MAX)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_i32(i32::MIN)
    );
}

/// Cf. TimeSpan.cs#L527, TimeSpanTests.cs#L591-L598
/// (`FromMinutes_Int_Single_ShouldCreate`)
#[test]
fn from_minutes_i64_basic() {
    assert_eq!(TimeSpan::from_hms(0, 0, 0), TimeSpan::from_minutes_i64(0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_MINUTE)),
        TimeSpan::from_minutes_i64(1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_MINUTE)),
        TimeSpan::from_minutes_i64(-1)
    );

    const MAX_MINUTES: i64 = 15_372_286_728;
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            MAX_MINUTES * TimeSpan::TICKS_PER_MINUTE
        )),
        TimeSpan::from_minutes_i64(MAX_MINUTES)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -MAX_MINUTES * TimeSpan::TICKS_PER_MINUTE
        )),
        TimeSpan::from_minutes_i64(-MAX_MINUTES)
    );
}

/// Cf. TimeSpan.cs#L433-L444 (`FromUnits`), TimeSpanTests.cs#L600-L606
/// (`FromMinutes_Int_Single_ShouldOverflow`)
#[test]
fn from_minutes_i64_overflow() {
    const MAX_MINUTES: i64 = 15_372_286_728;
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_i64(MAX_MINUTES + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_i64(-(MAX_MINUTES + 1))
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_i64(i64::MAX)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_i64(i64::MIN)
    );
}

/// Cf. TimeSpan.cs#L560, TimeSpanTests.cs#L646-L653
/// (`FromSeconds_Int_Single_ShouldCreate`)
#[test]
fn from_seconds_i64_basic() {
    assert_eq!(TimeSpan::from_hms(0, 0, 0), TimeSpan::from_seconds_i64(0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND)),
        TimeSpan::from_seconds_i64(1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_SECOND)),
        TimeSpan::from_seconds_i64(-1)
    );

    const MAX_SECONDS: i64 = 922_337_203_685;
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            MAX_SECONDS * TimeSpan::TICKS_PER_SECOND
        )),
        TimeSpan::from_seconds_i64(MAX_SECONDS)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -MAX_SECONDS * TimeSpan::TICKS_PER_SECOND
        )),
        TimeSpan::from_seconds_i64(-MAX_SECONDS)
    );
}

/// Cf. TimeSpan.cs#L433-L444 (`FromUnits`), TimeSpanTests.cs#L655-L661
/// (`FromSeconds_Int_Single_ShouldOverflow`)
#[test]
fn from_seconds_i64_overflow() {
    const MAX_SECONDS: i64 = 922_337_203_685;
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_i64(MAX_SECONDS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_i64(-(MAX_SECONDS + 1))
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_i64(i64::MAX)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_i64(i64::MIN)
    );
}

/// Cf. TimeSpan.cs#L591-L592. C# has no dedicated single-argument
/// `FromMilliseconds_Int_Single_ShouldCreate` theory (only the two-argument
/// `milliseconds, microseconds` overload is tested directly), so bounds here mirror
/// the `internal const MinMilliseconds`/`MaxMilliseconds` comment values instead.
#[test]
fn from_milliseconds_i64_basic() {
    assert_eq!(
        TimeSpan::from_hms(0, 0, 0),
        TimeSpan::from_milliseconds_i64(0)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_MILLISECOND)),
        TimeSpan::from_milliseconds_i64(1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_MILLISECOND)),
        TimeSpan::from_milliseconds_i64(-1)
    );

    const MAX_MILLISECONDS: i64 = 922_337_203_685_477;
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            MAX_MILLISECONDS * TimeSpan::TICKS_PER_MILLISECOND
        )),
        TimeSpan::from_milliseconds_i64(MAX_MILLISECONDS)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -MAX_MILLISECONDS * TimeSpan::TICKS_PER_MILLISECOND
        )),
        TimeSpan::from_milliseconds_i64(-MAX_MILLISECONDS)
    );
}

/// Cf. TimeSpan.cs#L433-L444 (`FromUnits`)
#[test]
fn from_milliseconds_i64_overflow() {
    const MAX_MILLISECONDS: i64 = 922_337_203_685_477;
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_i64(MAX_MILLISECONDS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_i64(-(MAX_MILLISECONDS + 1))
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_i64(i64::MAX)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_i64(i64::MIN)
    );
}

/// Cf. TimeSpan.cs#L632 (`FromMicroseconds(long)`), TimeSpan.cs#L433-L444
/// (`FromUnits`). C# has no dedicated single-argument
/// `FromMicroseconds_Int_Single_ShouldCreate` theory (only the multi-component
/// `_parts` overloads are tested directly), so bounds here mirror the
/// `internal const MinMicroseconds`/`MaxMicroseconds` comment values instead.
#[test]
fn from_microseconds_i64_basic() {
    assert_eq!(
        TimeSpan::from_hms(0, 0, 0),
        TimeSpan::from_microseconds_i64(0)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_MICROSECOND)),
        TimeSpan::from_microseconds_i64(1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_MICROSECOND)),
        TimeSpan::from_microseconds_i64(-1)
    );

    const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            MAX_MICROSECONDS * TimeSpan::TICKS_PER_MICROSECOND
        )),
        TimeSpan::from_microseconds_i64(MAX_MICROSECONDS)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -MAX_MICROSECONDS * TimeSpan::TICKS_PER_MICROSECOND
        )),
        TimeSpan::from_microseconds_i64(-MAX_MICROSECONDS)
    );
}

/// Cf. TimeSpan.cs#L433-L444 (`FromUnits`)
#[test]
fn from_microseconds_i64_overflow() {
    const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_microseconds_i64(MAX_MICROSECONDS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_microseconds_i64(-(MAX_MICROSECONDS + 1))
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_microseconds_i64(i64::MAX)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_microseconds_i64(i64::MIN)
    );
}

/// Cf. TimeSpan.cs#L414, TimeSpan.cs#L636-L643, TimeSpanTests.cs#L770-L788 (`FromDays_TestData`, `FromDays`)
#[test]
fn from_days_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_DAY)),
        TimeSpan::from_days(1.0)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_days(0.0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_DAY / 2)),
        TimeSpan::from_days(0.5)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_DAY)),
        TimeSpan::from_days(-1.0)
    );
}

/// `TimeSpan.FromDays(double.NaN)` throws `ArgumentException` in C#.
///
/// Cf. TimeSpan.cs#L636-L643, TimeSpanTests.cs#L790-L800 (`FromDays_Invalid`)
#[test]
fn from_days_nan() {
    assert_eq!(
        Err(TimeSpanError::NotANumber),
        TimeSpan::from_days(f64::NAN)
    );
}

/// `TimeSpan.FromDays(double.PositiveInfinity)`/out-of-range values throw
/// `OverflowException` in C#.
///
/// Cf. TimeSpan.cs#L645-L656, TimeSpanTests.cs#L790-L800 (`FromDays_Invalid`)
#[test]
fn from_days_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days(f64::INFINITY)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days(f64::NEG_INFINITY)
    );
    assert_eq!(Err(TimeSpanError::Overflow), TimeSpan::from_days(1e300));
    assert_eq!(Err(TimeSpanError::Overflow), TimeSpan::from_days(-1e300));
}

/// The double/tick boundary: `MaxTicks` (`i64::MAX`) isn't exactly representable as
/// `f64` and rounds up to `2^63`, so C# special-cases `ticks == MaxTicks` to return
/// `MaxValue` directly rather than truncating a value that's actually one tick past
/// the representable range.
///
/// Cf. TimeSpan.cs#L651-L654
#[test]
fn from_days_max_ticks_boundary() {
    let value = i64::MAX as f64 / TimeSpan::TICKS_PER_DAY as f64;
    assert_eq!(Ok(TimeSpan::MAX), TimeSpan::from_days(value));
}

/// Cf. TimeSpan.cs#L634, TimeSpan.cs#L636-L643, TimeSpanTests.cs#L800-L814 (`FromHours_TestData`, `FromHours`)
#[test]
fn from_hours_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR)),
        TimeSpan::from_hours(1.0)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_hours(0.0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_HOUR)),
        TimeSpan::from_hours(-1.0)
    );
}

/// `TimeSpan.FromHours(double.NaN)` throws `ArgumentException` in C#.
///
/// Cf. TimeSpan.cs#L636-L643, TimeSpanTests.cs#L822-L832 (`FromHours_Invalid`)
#[test]
fn from_hours_nan() {
    assert_eq!(
        Err(TimeSpanError::NotANumber),
        TimeSpan::from_hours(f64::NAN)
    );
}

/// Cf. TimeSpan.cs#L645-L656, TimeSpanTests.cs#L822-L832 (`FromHours_Invalid`)
#[test]
fn from_hours_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours(f64::INFINITY)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours(f64::NEG_INFINITY)
    );
}

/// Cf. TimeSpan.cs#L651-L654
#[test]
fn from_hours_max_ticks_boundary() {
    let value = i64::MAX as f64 / TimeSpan::TICKS_PER_HOUR as f64;
    assert_eq!(Ok(TimeSpan::MAX), TimeSpan::from_hours(value));
}

/// Cf. TimeSpan.cs#L527, TimeSpan.cs#L636-L643, TimeSpanTests.cs#L832-L847 (`FromMinutes_TestData`, `FromMinutes`)
#[test]
fn from_minutes_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_MINUTE)),
        TimeSpan::from_minutes(1.0)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_minutes(0.0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_MINUTE)),
        TimeSpan::from_minutes(-1.0)
    );
}

/// `TimeSpan.FromMinutes(double.NaN)` throws `ArgumentException` in C#.
///
/// Cf. TimeSpan.cs#L636-L643, TimeSpanTests.cs#L855-L864 (`FromMinutes_Invalid`)
#[test]
fn from_minutes_nan() {
    assert_eq!(
        Err(TimeSpanError::NotANumber),
        TimeSpan::from_minutes(f64::NAN)
    );
}

/// Cf. TimeSpan.cs#L645-L656, TimeSpanTests.cs#L855-L864 (`FromMinutes_Invalid`)
#[test]
fn from_minutes_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes(f64::INFINITY)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes(f64::NEG_INFINITY)
    );
}

/// Cf. TimeSpan.cs#L651-L654
#[test]
fn from_minutes_max_ticks_boundary() {
    let value = i64::MAX as f64 / TimeSpan::TICKS_PER_MINUTE as f64;
    assert_eq!(Ok(TimeSpan::MAX), TimeSpan::from_minutes(value));
}

/// Cf. TimeSpan.cs#L560, TimeSpan.cs#L636-L643, TimeSpanTests.cs#L864-L879 (`FromSeconds_TestData`, `FromSeconds`)
#[test]
fn from_seconds_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND)),
        TimeSpan::from_seconds(1.0)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_seconds(0.0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_SECOND)),
        TimeSpan::from_seconds(-1.0)
    );
}

/// `TimeSpan.FromSeconds(double.NaN)` throws `ArgumentException` in C#.
///
/// Cf. TimeSpan.cs#L636-L643, TimeSpanTests.cs#L887-L896 (`FromSeconds_Invalid`)
#[test]
fn from_seconds_nan() {
    assert_eq!(
        Err(TimeSpanError::NotANumber),
        TimeSpan::from_seconds(f64::NAN)
    );
}

/// Cf. TimeSpan.cs#L645-L656, TimeSpanTests.cs#L887-L896 (`FromSeconds_Invalid`)
#[test]
fn from_seconds_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds(f64::INFINITY)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds(f64::NEG_INFINITY)
    );
}

/// Cf. TimeSpan.cs#L651-L654
#[test]
fn from_seconds_max_ticks_boundary() {
    let value = i64::MAX as f64 / TimeSpan::TICKS_PER_SECOND as f64;
    assert_eq!(Ok(TimeSpan::MAX), TimeSpan::from_seconds(value));
}

/// Cf. TimeSpan.cs#L658, TimeSpan.cs#L636-L643, TimeSpanTests.cs#L903-L917 (`FromMilliseconds_TestData_NetCore`, `FromMilliseconds_Netcore`)
#[test]
fn from_milliseconds_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_MILLISECOND)),
        TimeSpan::from_milliseconds(1.0)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_milliseconds(0.0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_MILLISECOND)),
        TimeSpan::from_milliseconds(-1.0)
    );
}

/// `TimeSpan.FromMilliseconds(double.NaN)` throws `ArgumentException` in C#.
///
/// Cf. TimeSpan.cs#L636-L643, TimeSpanTests.cs#L930-L939 (`FromMilliseconds_Invalid`)
#[test]
fn from_milliseconds_nan() {
    assert_eq!(
        Err(TimeSpanError::NotANumber),
        TimeSpan::from_milliseconds(f64::NAN)
    );
}

/// Cf. TimeSpan.cs#L645-L656, TimeSpanTests.cs#L930-L939 (`FromMilliseconds_Invalid`)
#[test]
fn from_milliseconds_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds(f64::INFINITY)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds(f64::NEG_INFINITY)
    );
}

/// Cf. TimeSpan.cs#L651-L654
#[test]
fn from_milliseconds_max_ticks_boundary() {
    let value = i64::MAX as f64 / TimeSpan::TICKS_PER_MILLISECOND as f64;
    assert_eq!(Ok(TimeSpan::MAX), TimeSpan::from_milliseconds(value));
}

/// Cf. TimeSpan.cs#L632, TimeSpan.cs#L679, TimeSpan.cs#L636-L643
#[test]
fn from_microseconds_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_MICROSECOND)),
        TimeSpan::from_microseconds(1.0)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_microseconds(0.0));
    assert_eq!(
        Ok(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_MICROSECOND)),
        TimeSpan::from_microseconds(-1.0)
    );
}

/// `TimeSpan.FromMicroseconds(double.NaN)` throws `ArgumentException` in C#.
///
/// Cf. TimeSpan.cs#L636-L643
#[test]
fn from_microseconds_nan() {
    assert_eq!(
        Err(TimeSpanError::NotANumber),
        TimeSpan::from_microseconds(f64::NAN)
    );
}

/// Cf. TimeSpan.cs#L645-L656
#[test]
fn from_microseconds_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_microseconds(f64::INFINITY)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_microseconds(f64::NEG_INFINITY)
    );
}

/// Cf. TimeSpan.cs#L651-L654
#[test]
fn from_microseconds_max_ticks_boundary() {
    let value = i64::MAX as f64 / TimeSpan::TICKS_PER_MICROSECOND as f64;
    assert_eq!(Ok(TimeSpan::MAX), TimeSpan::from_microseconds(value));
}

// --- Multi-component `_parts` factories: `FromDays`/`FromHours`/`FromMinutes`/
// `FromSeconds`/`FromMilliseconds`'s overloads with optional trailing parameters,
// all delegating to the private `FromMicroseconds(Int128)` helper in C#. ---

/// Cf. TimeSpan.cs#L471-L481 (`FromDays` 6-arg overload), TimeSpanTests.cs#L353-372
/// (`FromDays_Int_Positive`/`FromDays_Int_Negative`/`FromDays_Int_Zero`)
#[test]
fn from_days_parts_basic() {
    assert_eq!(
        Ok(TimeSpan::from_dhms_micro(1, 2, 3, 4, 5, 6).unwrap()),
        TimeSpan::from_days_parts(1, 2, 3, 4, 5, 6)
    );
    assert_eq!(
        Ok(TimeSpan::from_dhms_micro(-1, -2, -3, -4, -5, -6).unwrap()),
        TimeSpan::from_days_parts(-1, -2, -3, -4, -5, -6)
    );
    assert_eq!(
        Ok(TimeSpan::ZERO),
        TimeSpan::from_days_parts(0, 0, 0, 0, 0, 0)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(TimeSpan::TICKS_PER_DAY)),
        TimeSpan::from_days_parts(1, 0, 0, 0, 0, 0)
    );
}

/// Two individually overflowing components with opposite sign should cancel out to
/// a result close to zero rather than erroring, verifying the sum is widened to a
/// wide-enough integer type before the range check — matching C#'s `Int128`-
/// accumulated `totalMicroseconds` (`Math.BigMul` per term).
///
/// Cf. TimeSpanTests.cs#L455-476
/// (`FromDays_Int_ShouldNotOverflow_WhenOverflowingParamIsCounteredByOppositeSignParam`)
#[test]
fn from_days_parts_opposite_sign_cancels_overflow() {
    const MAX_DAYS: i32 = 10_675_199;
    const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;

    let result =
        TimeSpan::from_days_parts(MAX_DAYS + 1, 0, 0, 0, 0, -(MAX_MICROSECONDS + 1)).unwrap();
    assert!(result > TimeSpan::from_days(-1.0).unwrap());
    assert!(result < TimeSpan::from_days(1.0).unwrap());
}

/// Cf. TimeSpanTests.cs#L484-511 (`FromDays_Int_ShouldOverflow`)
#[test]
fn from_days_parts_overflow() {
    const MAX_DAYS: i32 = 10_675_199;
    const MAX_HOURS: i32 = 256_204_778;
    const MAX_MINUTES: i64 = 15_372_286_728;
    const MAX_SECONDS: i64 = 922_337_203_685;
    const MAX_MILLISECONDS: i64 = 922_337_203_685_477;
    const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;

    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(MAX_DAYS + 1, 0, 0, 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(-(MAX_DAYS + 1), 0, 0, 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(0, MAX_HOURS + 1, 0, 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(0, 0, MAX_MINUTES + 1, 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(0, 0, 0, MAX_SECONDS + 1, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(0, 0, 0, 0, MAX_MILLISECONDS + 1, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(0, 0, 0, 0, 0, MAX_MICROSECONDS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(i32::MAX, i32::MAX, i64::MAX, i64::MAX, i64::MAX, i64::MAX)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days_parts(i32::MIN, i32::MIN, i64::MIN, i64::MIN, i64::MIN, i64::MIN)
    );
}

/// Cf. TimeSpan.cs#L507-L516 (`FromHours` 5-arg overload), TimeSpanTests.cs#L561-568
/// (`FromHours_Int_ShouldCreate`)
#[test]
fn from_hours_parts_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            TimeSpan::TICKS_PER_HOUR
                + TimeSpan::TICKS_PER_MINUTE
                + TimeSpan::TICKS_PER_SECOND
                + TimeSpan::TICKS_PER_MILLISECOND
                + TimeSpan::TICKS_PER_MICROSECOND
        )),
        TimeSpan::from_hours_parts(1, 1, 1, 1, 1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -(TimeSpan::TICKS_PER_HOUR
                + TimeSpan::TICKS_PER_MINUTE
                + TimeSpan::TICKS_PER_SECOND
                + TimeSpan::TICKS_PER_MILLISECOND
                + TimeSpan::TICKS_PER_MICROSECOND)
        )),
        TimeSpan::from_hours_parts(-1, -1, -1, -1, -1)
    );
    assert_eq!(
        Ok(TimeSpan::ZERO),
        TimeSpan::from_hours_parts(0, 0, 0, 0, 0)
    );
}

/// Cf. TimeSpanTests.cs#L570-591 (`FromHours_Int_ShouldOverflow`)
#[test]
fn from_hours_parts_overflow() {
    const MAX_HOURS: i32 = 256_204_778;
    const MAX_MINUTES: i64 = 15_372_286_728;
    const MAX_SECONDS: i64 = 922_337_203_685;
    const MAX_MILLISECONDS: i64 = 922_337_203_685_477;
    const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;

    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_parts(MAX_HOURS + 1, 0, 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_parts(-(MAX_HOURS + 1), 0, 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_parts(0, MAX_MINUTES + 1, 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_parts(0, 0, MAX_SECONDS + 1, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_parts(0, 0, 0, MAX_MILLISECONDS + 1, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_hours_parts(0, 0, 0, 0, MAX_MICROSECONDS + 1)
    );
}

/// Cf. TimeSpan.cs#L541-L549 (`FromMinutes` 4-arg overload), TimeSpanTests.cs#L619-627
/// (`FromMinutes_Int_ShouldCreate`)
#[test]
fn from_minutes_parts_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            TimeSpan::TICKS_PER_MINUTE
                + TimeSpan::TICKS_PER_SECOND
                + TimeSpan::TICKS_PER_MILLISECOND
                + TimeSpan::TICKS_PER_MICROSECOND
        )),
        TimeSpan::from_minutes_parts(1, 1, 1, 1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -(TimeSpan::TICKS_PER_MINUTE
                + TimeSpan::TICKS_PER_SECOND
                + TimeSpan::TICKS_PER_MILLISECOND
                + TimeSpan::TICKS_PER_MICROSECOND)
        )),
        TimeSpan::from_minutes_parts(-1, -1, -1, -1)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_minutes_parts(0, 0, 0, 0));
}

/// Cf. TimeSpanTests.cs#L629-637 (`FromMinutes_Int_ShouldOverflow`)
#[test]
fn from_minutes_parts_overflow() {
    const MAX_MINUTES: i64 = 15_372_286_728;
    const MAX_SECONDS: i64 = 922_337_203_685;
    const MAX_MILLISECONDS: i64 = 922_337_203_685_477;
    const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;

    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_parts(MAX_MINUTES + 1, 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_parts(-(MAX_MINUTES + 1), 0, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_parts(0, MAX_SECONDS + 1, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_parts(0, 0, MAX_MILLISECONDS + 1, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_parts(0, 0, 0, MAX_MICROSECONDS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_parts(i64::MAX, i64::MAX, i64::MAX, i64::MAX)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_minutes_parts(i64::MIN, i64::MIN, i64::MIN, i64::MIN)
    );
}

/// Cf. TimeSpan.cs#L573-L580 (`FromSeconds` 3-arg overload), TimeSpanTests.cs#L672-680
/// (`FromSeconds_Int_ShouldCreate`)
#[test]
fn from_seconds_parts_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            TimeSpan::TICKS_PER_SECOND
                + TimeSpan::TICKS_PER_MILLISECOND
                + TimeSpan::TICKS_PER_MICROSECOND
        )),
        TimeSpan::from_seconds_parts(1, 1, 1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -(TimeSpan::TICKS_PER_SECOND
                + TimeSpan::TICKS_PER_MILLISECOND
                + TimeSpan::TICKS_PER_MICROSECOND)
        )),
        TimeSpan::from_seconds_parts(-1, -1, -1)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_seconds_parts(0, 0, 0));
}

/// A naive `f64`-based conversion of `832` milliseconds could lose precision and
/// produce a slightly different tick count than the exact integer arithmetic C#
/// uses (`Math.BigMul`-widened `Int128`, never touching `f64`).
///
/// Cf. TimeSpanTests.cs#L374-379 (`FromSeconds_Int_ShouldGiveResultWithPrecision`,
/// citing https://github.com/dotnet/runtime/issues/93890)
#[test]
fn from_seconds_parts_precision() {
    assert_eq!(
        Ok(TimeSpan::from_dhms_micro(0, 0, 0, 101, 832, 0).unwrap()),
        TimeSpan::from_seconds_parts(101, 832, 0)
    );
}

/// Cf. TimeSpanTests.cs#L682-696 (`FromSeconds_Int_ShouldOverflow`)
#[test]
fn from_seconds_parts_overflow() {
    const MAX_SECONDS: i64 = 922_337_203_685;
    const MAX_MILLISECONDS: i64 = 922_337_203_685_477;
    const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;

    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_parts(MAX_SECONDS + 1, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_parts(-(MAX_SECONDS + 1), 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_parts(0, MAX_MILLISECONDS + 1, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_parts(0, 0, MAX_MICROSECONDS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_parts(i64::MAX, 0, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_parts(0, i64::MAX, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_seconds_parts(0, 0, i64::MAX)
    );
}

/// Cf. TimeSpan.cs#L604-L610 (`FromMilliseconds` 2-arg overload). Also covers what
/// #52/#54/#55 describe (`from_days_parts`/`from_minutes_parts`/`from_seconds_parts`)
/// and, like them, follows the same `FromMicroseconds(Int128)`-delegating pattern as
/// the other `_parts` factories above, so it's fixed alongside them here even though
/// no issue named it individually.
///
/// Cf. TimeSpanTests.cs#L710-724 (`FromMilliseconds_Int_ShouldCreate`)
#[test]
fn from_milliseconds_parts_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            TimeSpan::TICKS_PER_MILLISECOND + TimeSpan::TICKS_PER_MICROSECOND
        )),
        TimeSpan::from_milliseconds_parts(1, 1)
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(
            -(TimeSpan::TICKS_PER_MILLISECOND + TimeSpan::TICKS_PER_MICROSECOND)
        )),
        TimeSpan::from_milliseconds_parts(-1, -1)
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::from_milliseconds_parts(0, 0));
}

/// Cf. TimeSpanTests.cs#L727-747 (`FromMilliseconds_Int_ShouldOverflow`)
#[test]
fn from_milliseconds_parts_overflow() {
    const MAX_MILLISECONDS: i64 = 922_337_203_685_477;
    const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;

    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_parts(MAX_MILLISECONDS + 1, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_parts(-(MAX_MILLISECONDS + 1), 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_parts(0, MAX_MICROSECONDS + 1)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_parts(0, -(MAX_MICROSECONDS + 1))
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_parts(i64::MAX, 0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_milliseconds_parts(0, i64::MAX)
    );
}
