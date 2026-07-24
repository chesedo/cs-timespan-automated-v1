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
