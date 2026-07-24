//! Initial batch of ported tests, covering only the narrow core implemented so far
//! (construction from ticks, component accessors, `Total*` accessors, and derived
//! equality/ordering). Not full parity with TimeSpanTests.cs yet — see AGENTS.md's
//! test coverage-parity rule; the rest lands via the drift-scan/work-issue loop as
//! more of the C# surface gets real implementations.

use cs_timespan_automated_v1::TimeSpan;

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
