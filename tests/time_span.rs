//! Initial batch of ported tests, covering only the narrow core implemented so far
//! (construction from ticks, component accessors, `Total*` accessors, and derived
//! equality/ordering). Not full parity with TimeSpanTests.cs yet — see AGENTS.md's
//! test coverage-parity rule; the rest lands via the drift-scan/work-issue loop as
//! more of the C# surface gets real implementations.

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

/// `checked_add` performs real tick addition and only reports overflow via the
/// two's-complement sign-bit check that `operator+` uses (TimeSpan.cs#L893-L905),
/// not unconditionally.
///
/// Cf. TimeSpanTests.cs (`Add`), TimeSpan.cs#L389, #L893-L905
#[test]
fn checked_add_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(2)),
        TimeSpan::from_ticks(1).checked_add(TimeSpan::from_ticks(1))
    );
}

/// `TimeSpan.MaxValue + TimeSpan.FromTicks(1)` throws `OverflowException` in C#
/// because the result's sign flips relative to two identically-signed operands.
///
/// Cf. TimeSpan.cs#L893-L905
#[test]
fn checked_add_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::MAX.checked_add(TimeSpan::from_ticks(1))
    );
}

/// `TimeSpan.MaxValue + TimeSpan.MinValue` does NOT throw in C#: the operands have
/// opposite signs, so the sign-bit overflow check never triggers, and the true
/// two's-complement sum (`-1` tick) is returned.
///
/// Cf. TimeSpan.cs#L893-L905
#[test]
fn checked_add_opposite_signs_no_overflow() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(-1)),
        TimeSpan::MAX.checked_add(TimeSpan::MIN)
    );
}

/// Cf. TimeSpanTests.cs (`Subtract`), TimeSpan.cs#L687, #L877-L889
#[test]
fn checked_sub_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(2)),
        TimeSpan::from_ticks(5).checked_sub(TimeSpan::from_ticks(3))
    );
}

/// `TimeSpan.MinValue - TimeSpan.FromTicks(1)` throws `OverflowException` in C#.
///
/// Cf. TimeSpan.cs#L877-L889
#[test]
fn checked_sub_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::MIN.checked_sub(TimeSpan::from_ticks(1))
    );
}

/// `TimeSpan.MaxValue - TimeSpan.MinValue` DOES throw in C#: the operands have
/// different signs and the two's-complement result's sign is opposite `t1`'s,
/// which is exactly what the subtraction overflow check flags.
///
/// Cf. TimeSpan.cs#L877-L889
#[test]
fn checked_sub_different_signs_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::MAX.checked_sub(TimeSpan::MIN)
    );
}

/// `operator+` mirrors `checked_add` for the non-overflowing case.
///
/// Cf. TimeSpan.cs#L893-L905
#[test]
fn add_operator_basic() {
    assert_eq!(
        TimeSpan::from_ticks(2),
        TimeSpan::from_ticks(1) + TimeSpan::from_ticks(1)
    );
}

/// `operator+` throws `OverflowException` in C#; Rust's `Add` trait can't return a
/// `Result`, so the established pattern in this crate is to panic instead, mirroring
/// the C# exception at the operator layer while `checked_add` stays the fallible API.
///
/// Cf. TimeSpan.cs#L893-L905
#[test]
#[should_panic]
fn add_operator_overflow_panics() {
    let _ = TimeSpan::MAX + TimeSpan::from_ticks(1);
}

/// `operator-` mirrors `checked_sub` for the non-overflowing case.
///
/// Cf. TimeSpan.cs#L877-L889
#[test]
fn sub_operator_basic() {
    assert_eq!(
        TimeSpan::from_ticks(2),
        TimeSpan::from_ticks(5) - TimeSpan::from_ticks(3)
    );
}

/// `operator-` throws `OverflowException` in C#; ported as a panic for the same
/// reason `operator+` is (see `add_operator_overflow_panics`).
///
/// Cf. TimeSpan.cs#L877-L889
#[test]
#[should_panic]
fn sub_operator_overflow_panics() {
    let _ = TimeSpan::MIN - TimeSpan::from_ticks(1);
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

/// Cf. TimeSpanTests.cs#L1006-L1104 (`Parse_Valid_TestData`/`Parse`), restricted to the
/// `provider: null`/`CultureInfo.InvariantCulture` rows — culture-specific rows (e.g. the
/// `hr-HR` comma-decimal case) are out of scope for invariant-only `from_str`.
///
/// Expected tick counts were cross-checked against a live `dotnet100csharpcoreclr` run of
/// `TimeSpan.Parse(input, CultureInfo.InvariantCulture).Ticks` for every row below, not just
/// hand-derived from the day/hour/minute/second/ms constants.
#[test]
fn parse_valid() {
    // (input, expected ticks, whether "-" + input is also expected to parse to the negation
    // — false only for the leading-whitespace rows, mirroring the C# test's
    // `!char.IsWhiteSpace(input[0])` guard on the negation assertion)
    let cases: [(&str, i64, bool); 38] = [
        ("       12:24:02", 446_420_000_000, false),
        ("12:24:02      ", 446_420_000_000, true),
        ("     12:24:02      ", 446_420_000_000, false),
        ("0", 0, true),
        ("12:24", 446_400_000_000, true),
        ("12:24:02", 446_420_000_000, true),
        ("12.03:04", 10_478_400_000_000, true),
        ("12:24:02.01", 446_420_100_000, true),
        ("1:1:1.0", 36_610_000_000, true),
        ("1:1:1.0000000", 36_610_000_000, true),
        ("1:1:1.1", 36_611_000_000, true),
        ("1:1:1.01", 36_610_100_000, true),
        ("1:1:1.001", 36_610_010_000, true),
        ("1:1:1.0001", 36_610_001_000, true),
        ("1:1:1.00001", 36_610_000_100, true),
        ("1:1:1.000001", 36_610_000_010, true),
        ("1:1:1.0000001", 36_610_000_001, true),
        ("1.12:24:02", 1_310_420_000_000, true),
        ("1:12:24:02", 1_310_420_000_000, true),
        ("01.23:45:.67", 1_719_006_700_000, true),
        ("1.12:24:02.999", 1_310_429_990_000, true),
        ("1:1:.1", 36_601_000_000, true),
        ("1:1:.01", 36_600_100_000, true),
        ("1:1:.001", 36_600_010_000, true),
        ("1:1:.0001", 36_600_001_000, true),
        ("1:1:.00001", 36_600_000_100, true),
        ("1:1:.000001", 36_600_000_010, true),
        ("1:1:.0000001", 36_600_000_001, true),
        ("10675199", 9_223_371_936_000_000_000, true),
        ("10675199:00:00", 9_223_371_936_000_000_000, true),
        ("10675199:02:00:00", 9_223_372_008_000_000_000, true),
        ("10675199:02:48:00", 9_223_372_036_800_000_000, true),
        ("10675199:02:48:05", 9_223_372_036_850_000_000, true),
        ("10675199:02:48:05.4775", 9_223_372_036_854_775_000, true),
        ("00:00:59", 590_000_000, true),
        ("00:59:00", 35_400_000_000, true),
        ("23:00:00", 828_000_000_000, true),
        ("24:00:00", 20_736_000_000_000, true),
    ];

    for (input, expected_ticks, negatable) in cases {
        let expected = TimeSpan::from_ticks(expected_ticks);
        assert_eq!(Ok(expected), input.parse::<TimeSpan>(), "parsing {input:?}");

        if negatable {
            let negated = format!("-{input}");
            assert_eq!(
                Ok(TimeSpan::from_ticks(-expected_ticks)),
                negated.parse::<TimeSpan>(),
                "parsing {negated:?}"
            );
        }
    }
}

/// Cf. TimeSpanTests.cs#L1106-L1160 (`Parse_Invalid_TestData`/`Parse_Invalid`), restricted to
/// the `provider: null` rows minus the `null` input case (no `&str` equivalent of a null
/// `string` to parse) and the `hr-HR` culture-specific row.
#[test]
fn parse_invalid() {
    let cases: [(&str, TimeSpanError); 24] = [
        ("", TimeSpanError::InvalidFormat),
        ("-", TimeSpanError::InvalidFormat),
        ("garbage", TimeSpanError::InvalidFormat),
        ("12/12/12", TimeSpanError::InvalidFormat),
        ("00:", TimeSpanError::InvalidFormat),
        ("00:00:-01", TimeSpanError::InvalidFormat),
        ("\u{0}12:34:56", TimeSpanError::InvalidFormat),
        ("1\u{0}2:34:56", TimeSpanError::InvalidFormat),
        ("12\u{0}:34:56", TimeSpanError::InvalidFormat),
        ("00:00::00", TimeSpanError::InvalidFormat),
        ("00:00:00:", TimeSpanError::InvalidFormat),
        ("00:00:00:00:00:00:00:00", TimeSpanError::InvalidFormat),
        ("1:1:1.99999999", TimeSpanError::Overflow),
        ("2147483647", TimeSpanError::Overflow),
        ("2147483648", TimeSpanError::Overflow),
        ("10675200", TimeSpanError::Overflow),
        ("10675200:00:00", TimeSpanError::Overflow),
        ("10675199:03:00:00", TimeSpanError::Overflow),
        ("10675199:02:49:00", TimeSpanError::Overflow),
        ("10675199:02:48:06", TimeSpanError::Overflow),
        ("-10675199:02:48:06", TimeSpanError::Overflow),
        ("10675199:02:48:05.4776", TimeSpanError::Overflow),
        ("-10675199:02:48:05.4776", TimeSpanError::Overflow),
        ("00:00:60", TimeSpanError::Overflow),
    ];

    for (input, expected_err) in cases {
        assert_eq!(
            Err(expected_err),
            input.parse::<TimeSpan>(),
            "parsing {input:?}"
        );
    }

    // "00:60:00" and "24:00" (overflowing minutes/hours) are also part of the upstream
    // data set but share a row type with the table above; kept separate only because
    // Rust doesn't need the array to be homogeneous in any special way — listed here for
    // strict 1:1 parity with the upstream rows rather than folding them silently into the
    // table above out of order.
    assert_eq!(
        Err(TimeSpanError::Overflow),
        "00:60:00".parse::<TimeSpan>(),
        "parsing \"00:60:00\""
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        "24:00".parse::<TimeSpan>(),
        "parsing \"24:00\""
    );
}

/// Cf. TimeSpanTests.cs#L1730-L1752 (`ParseDifferentLengthFractionWithLeadingZerosData`/
/// `ParseDifferentLengthFractionWithLeadingZeros`), `Parse` half only — the `ParseExact(..,
/// "g", ..)` half is out of scope for invariant-only `from_str`.
#[test]
fn parse_different_length_fraction_with_leading_zeros() {
    let cases: [(&str, i64); 11] = [
        ("00:00:00.00000001", 0),
        ("00:00:00.00000005", 1),
        ("00:00:00.09999999", 1_000_000),
        ("00:00:00.0268435455", 268_435),
        ("00:00:00.01", 100_000),
        ("0:00:00.01000000", 100_000),
        ("0:00:00.010000000", 100_000),
        ("0:00:00.0123456", 123_456),
        ("0:00:00.00123456", 12_346),
        ("0:00:00.00000098", 10),
        ("0:00:00.00000099", 10),
    ];

    for (input, expected_ticks) in cases {
        assert_eq!(
            Ok(TimeSpan::from_ticks(expected_ticks)),
            input.parse::<TimeSpan>(),
            "parsing {input:?}"
        );
    }
}

/// `checked_neg` mirrors C#'s instance `Negate()`, which delegates to `operator-`
/// (TimeSpan.cs#L683, #L868-L875).
///
/// Cf. TimeSpanTests.cs#L982-L997 (`Negate`)
#[test]
fn checked_neg_basic() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(-12345)),
        TimeSpan::from_ticks(12345).checked_neg()
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(12345)),
        TimeSpan::from_ticks(-12345).checked_neg()
    );
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::ZERO.checked_neg());
}

/// `TimeSpan.MinValue.Negate()` throws `OverflowException` in C#, because negating
/// `long.MinValue` overflows `i64`.
///
/// Cf. TimeSpan.cs#L868-L875, TimeSpanTests.cs#L999-L1004 (`Negate_Invalid`)
#[test]
fn checked_neg_overflow() {
    assert_eq!(Err(TimeSpanError::Overflow), TimeSpan::MIN.checked_neg());
}

/// `operator-` (unary) mirrors `checked_neg` for the non-overflowing case.
///
/// Cf. TimeSpan.cs#L868-L875, TimeSpanTests.cs#L982-L997 (`Negate`)
#[test]
fn neg_operator_basic() {
    assert_eq!(TimeSpan::from_ticks(-5), -TimeSpan::from_ticks(5));
}

/// Unary `-TimeSpan.MinValue` throws `OverflowException` in C#; Rust's `Neg` trait
/// can't return a `Result`, so the established pattern in this crate is to panic
/// instead (see `add_operator_overflow_panics`).
///
/// Cf. TimeSpan.cs#L868-L875, TimeSpanTests.cs#L999-L1004 (`Negate_Invalid`)
#[test]
#[should_panic(expected = "overflowed its representable range")]
fn neg_operator_overflow_panics() {
    let _ = -TimeSpan::MIN;
}

/// `duration()` mirrors C#'s `Duration()`, returning the absolute tick count.
///
/// Cf. TimeSpanTests.cs#L276-L291 (`Duration`)
#[test]
fn duration_basic() {
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::ZERO.duration());
    assert_eq!(
        Ok(TimeSpan::from_ticks(12345)),
        TimeSpan::from_ticks(12345).duration()
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(12345)),
        TimeSpan::from_ticks(-12345).duration()
    );
}

/// `TimeSpan.MinValue.Duration()` throws `OverflowException` in C#, because taking
/// the absolute value of `long.MinValue` overflows `i64`.
///
/// Cf. TimeSpan.cs#L416-L423, TimeSpanTests.cs#L292-L297 (`Duration_Invalid`)
#[test]
fn duration_overflow() {
    assert_eq!(Err(TimeSpanError::Overflow), TimeSpan::MIN.duration());
}

/// `TimeSpan / TimeSpan` divides ticks as `f64`, matching C#'s
/// `t1.Ticks / (double)t2.Ticks`.
///
/// Cf. TimeSpan.cs#L936-L941 (`operator /(TimeSpan, TimeSpan)`)
#[test]
fn divide_time_span_operator_basic() {
    assert_eq!(2.0, TimeSpan::from_ticks(10) / TimeSpan::from_ticks(5));
    assert_eq!(0.5, TimeSpan::from_ticks(5) / TimeSpan::from_ticks(10));
    assert_eq!(-2.0, TimeSpan::from_ticks(-10) / TimeSpan::from_ticks(5));
}

/// Deliberately infallible per the comment directly above the C# operator:
/// dividing a non-zero `TimeSpan` by `TimeSpan.Zero` is defined to yield
/// `+Infinity`/`-Infinity` rather than throwing.
///
/// Cf. TimeSpan.cs#L936-L941
#[test]
fn divide_time_span_operator_by_zero_yields_infinity() {
    assert_eq!(f64::INFINITY, TimeSpan::from_ticks(1) / TimeSpan::ZERO);
    assert_eq!(f64::NEG_INFINITY, TimeSpan::from_ticks(-1) / TimeSpan::ZERO);
}

/// `TimeSpan.Zero / TimeSpan.Zero` is defined to yield `NaN` per the same
/// comment, rather than throwing.
///
/// Cf. TimeSpan.cs#L936-L941
#[test]
fn divide_time_span_operator_zero_by_zero_yields_nan() {
    let result = TimeSpan::ZERO / TimeSpan::ZERO;
    assert!(result.is_nan());
}

/// `Divide(TimeSpan)` forwards to the same operator and thus has identical
/// infallible semantics.
///
/// Cf. TimeSpan.cs#L693 (instance `Divide(TimeSpan)`), TimeSpan.cs#L936-L941
#[test]
fn divide_time_span_method_matches_operator() {
    assert_eq!(
        2.0,
        TimeSpan::from_ticks(10).divide_time_span(TimeSpan::from_ticks(5))
    );
    assert_eq!(
        f64::INFINITY,
        TimeSpan::from_ticks(1).divide_time_span(TimeSpan::ZERO)
    );
    assert!(TimeSpan::ZERO.divide_time_span(TimeSpan::ZERO).is_nan());
}

/// `Display` mirrors C#'s parameterless `ToString()`, which delegates to
/// `TimeSpanFormat.FormatC` — the invariant, culture-independent constant "c" format
/// `[-][d.]hh:mm:ss[.fffffff]`. Only the constant-format rows of the C# test's
/// `ToString_TestData` are ported here (the `null`/`"c"`/`"t"`/`"T"` format-string rows,
/// which C# routes to the same `FormatC` path) — the `"g"`/`"G"` general-format rows and
/// the culture-aware overloads are out of scope; see the `Display` impl's doc comment.
///
/// Cf. TimeSpanTests.cs#L1539-L1591 (`ToString_TestData`, constant-format rows),
/// TimeSpanTests.cs#L1656-L1666 (`ToString_Valid`)
#[test]
fn display_constant_format() {
    assert_eq!(
        "142.21:21:18.9101112",
        TimeSpan::from_ticks(123_456_789_101_112).to_string()
    );
    assert_eq!("00:00:00", TimeSpan::ZERO.to_string());
    assert_eq!("00:00:00.0000001", TimeSpan::from_ticks(1).to_string());
    assert_eq!("-00:00:00.0000001", TimeSpan::from_ticks(-1).to_string());
    assert_eq!("10675199.02:48:05.4775807", TimeSpan::MAX.to_string());
    assert_eq!("-10675199.02:48:05.4775808", TimeSpan::MIN.to_string());
    assert_eq!("01:02:03", TimeSpan::from_hms(1, 2, 3).unwrap().to_string());
    assert_eq!(
        "-01:02:03",
        (-TimeSpan::from_hms(1, 2, 3).unwrap()).to_string()
    );
    assert_eq!(
        "12:34:56",
        TimeSpan::from_hms(12, 34, 56).unwrap().to_string()
    );
    assert_eq!(
        "13.10:56:23",
        TimeSpan::from_dhms(12, 34, 56, 23).unwrap().to_string()
    );
    assert_eq!(
        "13.10:56:23.0450000",
        TimeSpan::from_dhms_milli(12, 34, 56, 23, 45)
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "23:59:59.9990000",
        TimeSpan::from_dhms_milli(0, 23, 59, 59, 999)
            .unwrap()
            .to_string()
    );
}
