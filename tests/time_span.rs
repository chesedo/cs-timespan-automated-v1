//! Initial batch of ported tests, covering only the narrow core implemented so far
//! (construction from ticks, component accessors, `Total*` accessors, and derived
//! equality/ordering). Not full parity with TimeSpanTests.cs yet — see AGENTS.md's
//! test coverage-parity rule; the rest lands via the drift-scan/work-issue loop as
//! more of the C# surface gets real implementations.

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError, TimeSpanStyles};

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

/// Cf. TimeSpanTests.cs#L1162-L1206 (`ParseExact_Valid_TestData`), restricted to the
/// "Custom timespan formats" rows (TimeSpanTests.cs#L1191-L1205) — the standard
/// single-letter-format rows ("c"/"t"/"T"/"g"/"G", TimeSpanTests.cs#L1164-L1189) are out of
/// scope for `TimeSpan::parse_exact`'s narrow custom-format-string-only slice; see its doc
/// comment.
#[test]
fn parse_exact_valid() {
    let cases: [(&str, &str, TimeSpan); 14] = [
        (
            "12.23:32:43",
            r"dd\.h\:m\:s",
            TimeSpan::from_dhms(12, 23, 32, 43).unwrap(),
        ),
        (
            "012.23:32:43.893",
            r"ddd\.h\:m\:s\.fff",
            TimeSpan::from_dhms_milli(12, 23, 32, 43, 893).unwrap(),
        ),
        (
            "12.05:02:03",
            r"d\.hh\:mm\:ss",
            TimeSpan::from_dhms(12, 5, 2, 3).unwrap(),
        ),
        (
            "12:34 minutes",
            r"mm\:ss\ \m\i\n\u\t\e\s",
            TimeSpan::from_hms(0, 12, 34).unwrap(),
        ),
        (
            "12:34 minutes",
            r#"mm\:ss\ "minutes""#,
            TimeSpan::from_hms(0, 12, 34).unwrap(),
        ),
        (
            "12:34 minutes",
            r"mm\:ss\ 'minutes'",
            TimeSpan::from_hms(0, 12, 34).unwrap(),
        ),
        (
            "678",
            "fff",
            TimeSpan::from_dhms_milli(0, 0, 0, 0, 678).unwrap(),
        ),
        (
            "678",
            "FFF",
            TimeSpan::from_dhms_milli(0, 0, 0, 0, 678).unwrap(),
        ),
        ("3", "%d", TimeSpan::from_dhms(3, 0, 0, 0).unwrap()),
        ("3", "%h", TimeSpan::from_hms(3, 0, 0).unwrap()),
        ("3", "%m", TimeSpan::from_hms(0, 3, 0).unwrap()),
        ("3", "%s", TimeSpan::from_hms(0, 0, 3).unwrap()),
        (
            "3",
            "%f",
            TimeSpan::from_dhms_milli(0, 0, 0, 0, 300).unwrap(),
        ),
        (
            "3",
            "%F",
            TimeSpan::from_dhms_milli(0, 0, 0, 0, 300).unwrap(),
        ),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::None),
            "parsing {input:?} against format {format:?}"
        );
    }
}

/// Cf. TimeSpanTests.cs#L1230-L1241 (`ParseExact`'s `TimeSpanStyles.AssumeNegative`
/// assertion — gated there on `format` not being one of the five standard single-letter
/// formats, so only exercised here against a sample of the custom-format rows from
/// `parse_exact_valid` above).
#[test]
fn parse_exact_assume_negative() {
    let cases: [(&str, &str, TimeSpan); 3] = [
        (
            "12.23:32:43",
            r"dd\.h\:m\:s",
            TimeSpan::from_dhms(12, 23, 32, 43).unwrap(),
        ),
        ("3", "%h", TimeSpan::from_hms(3, 0, 0).unwrap()),
        (
            "678",
            "fff",
            TimeSpan::from_dhms_milli(0, 0, 0, 0, 678).unwrap(),
        ),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(-expected),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::AssumeNegative),
            "parsing {input:?} against format {format:?} with AssumeNegative"
        );
    }
}

/// Cf. TimeSpanTests.cs#L1252-L1304 (`ParseExact_Invalid_TestData`), restricted to rows
/// usable without a `null` `string`/`string[]` (no `&str` equivalent) and to the
/// format-agnostic `""`/`"garbage"`-style rows plus the "Custom timespan formats" section
/// (TimeSpanTests.cs#L1275-L1303) — the standard single-letter-format rows
/// (TimeSpanTests.cs#L1261-L1274, `"c"`/`"g"`/`"G"`) are out of scope for
/// `TimeSpan::parse_exact`'s narrow custom-format-string-only slice: a 1-character `format`
/// unconditionally returns `InvalidFormat` here regardless of what the real C# algorithm
/// for that particular standard format would have done with the input, so those rows would
/// only coincidentally match (or not) rather than actually exercise this port's algorithm.
#[test]
fn parse_exact_invalid() {
    let cases: [(&str, &str, TimeSpanError); 28] = [
        ("00:00:00", "", TimeSpanError::InvalidFormat),
        ("12.5:2", "V", TimeSpanError::InvalidFormat),
        ("12.35:32:43", r"dd\.h\:m\:s", TimeSpanError::Overflow),
        ("12.5:2:3", r"d\.hh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("12.5:2", r"d\.hh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("678", "ffff", TimeSpanError::InvalidFormat),
        ("00000012", "FFFFFFFF", TimeSpanError::InvalidFormat),
        ("12:034:56", r"hh\mm\ss", TimeSpanError::InvalidFormat),
        ("12:34:056", r"hh\mm\ss", TimeSpanError::InvalidFormat),
        (
            "12:34 minutes",
            r#"mm\:ss\ "minutes"#,
            TimeSpanError::InvalidFormat,
        ),
        (
            "12:34 minutes",
            r"mm\:ss\ 'minutes",
            TimeSpanError::InvalidFormat,
        ),
        (
            "12:34 mints",
            r#"mm\:ss\ "minutes""#,
            TimeSpanError::InvalidFormat,
        ),
        (
            "12:34 mints",
            r"mm\:ss\ 'minutes'",
            TimeSpanError::InvalidFormat,
        ),
        ("1", "d%", TimeSpanError::InvalidFormat),
        ("1", "%%d", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hhh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:hh\:ss", TimeSpanError::InvalidFormat),
        ("123:34:56", r"hh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:mmm\:ss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:mm\:mm", TimeSpanError::InvalidFormat),
        ("12:345:56", r"hh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:mm\:sss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:ss\:ss", TimeSpanError::InvalidFormat),
        ("12:45", "ff:ff", TimeSpanError::InvalidFormat),
        ("000000123", "ddddddddd", TimeSpanError::InvalidFormat),
        ("12:34:56", "dd:dd:hh", TimeSpanError::InvalidFormat),
        ("123:45", "dd:hh", TimeSpanError::InvalidFormat),
        ("12:34", "dd:vv", TimeSpanError::InvalidFormat),
    ];

    for (input, format, expected_err) in cases {
        assert_eq!(
            Err(expected_err),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::None),
            "parsing {input:?} against format {format:?}"
        );
    }
}

/// Cf. TimeSpanTests.cs#L1163-L1168 (`ParseExact_Valid_TestData`'s `"c"`/`"t"`/`"T"` rows —
/// all three characters dispatch to the exact same `TryParseTimeSpanConstant` algorithm
/// upstream, TimeSpanParse.cs#L1237-1239, hence looping over all three here too).
#[test]
fn parse_exact_standard_constant_valid() {
    let cases: [(&str, TimeSpan); 3] = [
        ("12:24:02", TimeSpan::from_hms(12, 24, 2).unwrap()),
        ("1.12:24:02", TimeSpan::from_dhms(1, 12, 24, 2).unwrap()),
        (
            "-01.07:45:16.999",
            -TimeSpan::from_dhms_milli(1, 7, 45, 16, 999).unwrap(),
        ),
    ];

    for format in ["c", "t", "T"] {
        for (input, expected) in cases {
            assert_eq!(
                Ok(expected),
                TimeSpan::parse_exact(input, format, TimeSpanStyles::None),
                "parsing {input:?} against format {format:?}"
            );
        }
    }
}

/// Cf. TimeSpanTests.cs#L1170-L1183 (`ParseExact_Valid_TestData`'s `"g"` rows).
#[test]
fn parse_exact_standard_g_valid() {
    let cases: [(&str, TimeSpan); 13] = [
        ("12", TimeSpan::from_dhms(12, 0, 0, 0).unwrap()),
        ("-12", -TimeSpan::from_dhms(12, 0, 0, 0).unwrap()),
        ("12:34", TimeSpan::from_hms(12, 34, 0).unwrap()),
        ("-12:34", -TimeSpan::from_hms(12, 34, 0).unwrap()),
        (
            "1:2:.3",
            TimeSpan::from_dhms_milli(0, 1, 2, 0, 300).unwrap(),
        ),
        (
            "-1:2:.3",
            -TimeSpan::from_dhms_milli(0, 1, 2, 0, 300).unwrap(),
        ),
        ("12:24:02", TimeSpan::from_hms(12, 24, 2).unwrap()),
        (
            "12:24:02.123",
            TimeSpan::from_dhms_milli(0, 12, 24, 2, 123).unwrap(),
        ),
        (
            "-12:24:02.123",
            -TimeSpan::from_dhms_milli(0, 12, 24, 2, 123).unwrap(),
        ),
        (
            "1:2:3:.4",
            TimeSpan::from_dhms_milli(1, 2, 3, 0, 400).unwrap(),
        ),
        (
            "-1:2:3:.4",
            -TimeSpan::from_dhms_milli(1, 2, 3, 0, 400).unwrap(),
        ),
        ("1:12:24:02", TimeSpan::from_dhms(1, 12, 24, 2).unwrap()),
        (
            "-01:07:45:16.999",
            -TimeSpan::from_dhms_milli(1, 7, 45, 16, 999).unwrap(),
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact(input, "g", TimeSpanStyles::None),
            "parsing {input:?} against format \"g\""
        );
    }
}

/// Cf. TimeSpanTests.cs#L1185-L1188 (`ParseExact_Valid_TestData`'s `"G"` rows).
#[test]
fn parse_exact_standard_g_long_valid() {
    let cases: [(&str, TimeSpan); 2] = [
        (
            "1:12:24:02.243",
            TimeSpan::from_dhms_milli(1, 12, 24, 2, 243).unwrap(),
        ),
        (
            "-01:07:45:16.999",
            -TimeSpan::from_dhms_milli(1, 7, 45, 16, 999).unwrap(),
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact(input, "G", TimeSpanStyles::None),
            "parsing {input:?} against format \"G\""
        );
    }
}

/// `TimeSpanStyles` is interpreted only for custom formats, not the five standard
/// single-letter ones — C#'s dispatch (TimeSpanParse.cs#L1231-1241) never passes `styles`
/// into `TryParseTimeSpanConstant`/`TryParseTimeSpan` for `'c'`/`'t'`/`'T'`/`'g'`/`'G'`, so
/// `AssumeNegative` has no effect on them (cf. `TimeSpanTests.cs`'s `ParseExact` test,
/// which skips the `AssumeNegative` assertions entirely for these five formats,
/// TimeSpanTests.cs#L1225-1229).
#[test]
fn parse_exact_standard_ignores_styles() {
    // "G" only accepts the full "d:h:m:s.f" shape, unlike the other four, so it needs its
    // own representative input rather than sharing "12:24:02" with the rest.
    let cases: [(&str, &str, TimeSpan); 5] = [
        ("12:24:02", "c", TimeSpan::from_hms(12, 24, 2).unwrap()),
        ("12:24:02", "t", TimeSpan::from_hms(12, 24, 2).unwrap()),
        ("12:24:02", "T", TimeSpan::from_hms(12, 24, 2).unwrap()),
        ("12:24:02", "g", TimeSpan::from_hms(12, 24, 2).unwrap()),
        (
            "1:12:24:02.243",
            "G",
            TimeSpan::from_dhms_milli(1, 12, 24, 2, 243).unwrap(),
        ),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::AssumeNegative),
            "format {format:?} should ignore AssumeNegative"
        );
    }
}

/// Cf. TimeSpanTests.cs#L1252-L1274 (`ParseExact_Invalid_TestData`'s format-agnostic rows
/// plus the standard single-letter-format rows), restricted to rows usable without a
/// `null` `string` (no `&str` equivalent).
#[test]
fn parse_exact_standard_invalid() {
    let cases: [(&str, &str, TimeSpanError); 16] = [
        ("", "c", TimeSpanError::InvalidFormat),
        ("-", "c", TimeSpanError::InvalidFormat),
        ("garbage", "c", TimeSpanError::InvalidFormat),
        ("24:24:02", "c", TimeSpanError::Overflow),
        ("1:60:02", "c", TimeSpanError::Overflow),
        ("1:59:60", "c", TimeSpanError::Overflow),
        ("1.24:59:02", "c", TimeSpanError::Overflow),
        ("1.2:60:02", "c", TimeSpanError::Overflow),
        ("1?59:02", "c", TimeSpanError::InvalidFormat),
        ("1:59?02", "c", TimeSpanError::InvalidFormat),
        ("1:59:02?123", "c", TimeSpanError::InvalidFormat),
        ("1:12:24:02", "c", TimeSpanError::InvalidFormat),
        ("12:61:02", "g", TimeSpanError::Overflow),
        ("1.12:24:02", "g", TimeSpanError::InvalidFormat),
        ("1:07:45:16.99999999", "G", TimeSpanError::Overflow),
        ("1:12:24:02", "G", TimeSpanError::InvalidFormat),
    ];

    for (input, format, expected_err) in cases {
        assert_eq!(
            Err(expected_err),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::None),
            "parsing {input:?} against format {format:?}"
        );
    }
}

/// Cf. `TimeSpanTests.cs`'s `ParseExactTest_Valid` body (TimeSpanTests.cs#L1209-1234),
/// which re-asserts every `ParseExact_Valid_TestData` row against the single-format-wrapped-
/// in-an-array overload (`TimeSpan.ParseExact(input, new string[] { format }, ...)`) too —
/// a single-element array must behave identically to the plain single-format overload.
#[test]
fn parse_exact_multiple_single_format_matches_parse_exact() {
    let cases: [(&str, &str, TimeSpan); 5] = [
        (
            "12.23:32:43",
            r"dd\.h\:m\:s",
            TimeSpan::from_dhms(12, 23, 32, 43).unwrap(),
        ),
        ("3", "%h", TimeSpan::from_hms(3, 0, 0).unwrap()),
        (
            "678",
            "fff",
            TimeSpan::from_dhms_milli(0, 0, 0, 0, 678).unwrap(),
        ),
        (
            "1.12:24:02",
            "c",
            TimeSpan::from_dhms(1, 12, 24, 2).unwrap(),
        ),
        ("12:24:02", "g", TimeSpan::from_hms(12, 24, 2).unwrap()),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact_multiple(input, &[format], TimeSpanStyles::None),
            "parsing {input:?} against single-element formats array {format:?}"
        );
    }
}

/// Cf. `TimeSpanTests.cs`'s `ParseExactTest_Valid` body (TimeSpanTests.cs#L1234, #L1239):
/// `TimeSpanStyles.AssumeNegative` is honored the same way through the array overload as
/// through the single-format overload (for the custom-format-string rows it applies to).
#[test]
fn parse_exact_multiple_assume_negative() {
    let cases: [(&str, &str, TimeSpan); 2] = [
        (
            "12.23:32:43",
            r"dd\.h\:m\:s",
            TimeSpan::from_dhms(12, 23, 32, 43).unwrap(),
        ),
        ("3", "%h", TimeSpan::from_hms(3, 0, 0).unwrap()),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(-expected),
            TimeSpan::parse_exact_multiple(input, &[format], TimeSpanStyles::AssumeNegative),
            "parsing {input:?} against single-element formats array {format:?} with \
             AssumeNegative"
        );
    }
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): formats are tried in
/// array order, and the first one that matches wins. `"%h"` against `"3"` fails outright
/// (no `:` literal for `%h` to match against, so it's not just "wrong interpretation" —
/// `hh\:mm\:ss` requires two digits then a literal `:`, which a bare `"3"` doesn't have),
/// so the array must fall through to `"%h"` and succeed there.
#[test]
fn parse_exact_multiple_tries_formats_in_order() {
    assert_eq!(
        Ok(TimeSpan::from_hms(3, 0, 0).unwrap()),
        TimeSpan::parse_exact_multiple("3", &[r"hh\:mm\:ss", "%h"], TimeSpanStyles::None),
        "first format should fail to match, falling through to the second"
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): order-sensitivity is
/// also observable in *which* value results, not just whether parsing succeeds — `"%h"` and
/// `"%m"` both accept a single digit, but interpret it differently, so swapping which comes
/// first in the array changes the parsed result.
#[test]
fn parse_exact_multiple_first_match_determines_interpretation() {
    assert_eq!(
        Ok(TimeSpan::from_hms(3, 0, 0).unwrap()),
        TimeSpan::parse_exact_multiple("3", &["%h", "%m"], TimeSpanStyles::None),
    );
    assert_eq!(
        Ok(TimeSpan::from_hms(0, 3, 0).unwrap()),
        TimeSpan::parse_exact_multiple("3", &["%m", "%h"], TimeSpanStyles::None),
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): `formats.Length == 0`
/// is a distinct `SetNoFormatSpecifierFailure` bad-format failure — there's no `&str`
/// equivalent of C#'s separate `formats == null` -> `ArgumentNullException` case in this
/// crate (a `&[&str]` can't be null), but the empty-slice case still applies and, like every
/// other format failure in this crate, maps to `TimeSpanError::InvalidFormat`.
#[test]
fn parse_exact_multiple_empty_formats_array() {
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("12:34:56", &[], TimeSpanStyles::None),
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): an empty format string
/// anywhere in the array (`string.IsNullOrEmpty(format)`) is an immediate
/// `SetBadFormatSpecifierFailure`, returned right away rather than being skipped in favor of
/// a later entry that would otherwise have matched — this is the one case in the loop that
/// doesn't fall through to try the next format.
#[test]
fn parse_exact_multiple_empty_format_stops_immediately() {
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("3", &["", "%h"], TimeSpanStyles::None),
        "an empty format entry must fail immediately, not be skipped in favor of a later \
         match"
    );
}

/// Cf. `TimeSpanTests.cs`'s `ParseExactTest_Invalid` body (TimeSpanTests.cs#L1313-1315):
/// `exceptionTypeMultiple = exceptionType == typeof(OverflowException) ... ?
/// typeof(FormatException) : exceptionType` — `TryParseExactMultipleTimeSpan`'s per-format
/// attempts always run with `throwOnFailure: false` (a fresh, independent `TimeSpanResult`
/// each time), so an individual attempt's `OverflowException` is discarded exactly like an
/// individual attempt's `FormatException` would be; only the generic `SetBadTimeSpanFailure`
/// bad-format failure surfaces once every format in the array has failed. So even a
/// single-element array around a format that would overflow on its own turns that Overflow
/// into InvalidFormat here, unlike `TimeSpan::parse_exact` on the same input/format pair.
#[test]
fn parse_exact_multiple_overflow_becomes_invalid_format() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::parse_exact("12.35:32:43", r"dd\.h\:m\:s", TimeSpanStyles::None),
        "sanity check: the single-format overload reports Overflow directly"
    );
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("12.35:32:43", &[r"dd\.h\:m\:s"], TimeSpanStyles::None),
        "the array overload must not leak the inner Overflow — it's swallowed into the \
         generic bad-format failure once every format has failed"
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): when no format in the
/// array matches, the loop falls off the end into `SetBadTimeSpanFailure`.
#[test]
fn parse_exact_multiple_no_format_matches() {
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("garbage", &["%h", "%m", "%s"], TimeSpanStyles::None),
    );
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

/// `to_string_format` mirrors C#'s `ToString(string? format)` for the constant `"c"`
/// format and its `"t"`/`"T"` aliases, plus an empty/absent format string — all four
/// route to the same invariant output as `Display`. Only the invariant-culture rows of
/// the C# test's `ToString_TestData` constant-format block are ported (culture is
/// always ignored for `"c"`/`"t"`/`"T"`, matching `Display`'s existing scope).
///
/// Cf. TimeSpanTests.cs#L1572-L1591 (`ToString_TestData`, constant-format rows),
/// TimeSpanTests.cs#L1656-L1669 (`ToString_Valid`)
#[test]
fn to_string_format_constant() {
    let cases: &[(TimeSpan, &str)] = &[
        (
            TimeSpan::from_ticks(123_456_789_101_112),
            "142.21:21:18.9101112",
        ),
        (TimeSpan::ZERO, "00:00:00"),
        (TimeSpan::from_ticks(1), "00:00:00.0000001"),
        (TimeSpan::from_ticks(-1), "-00:00:00.0000001"),
        (TimeSpan::MAX, "10675199.02:48:05.4775807"),
        (TimeSpan::MIN, "-10675199.02:48:05.4775808"),
    ];

    for (input, expected) in cases {
        for format in ["", "c", "t", "T"] {
            assert_eq!(Ok((*expected).to_string()), input.to_string_format(format));
        }
    }
}

/// The general short `"g"` format: variable-width hours (one digit when `< 10`), the
/// day segment omitted entirely when zero, and a fraction shown only when non-zero
/// with trailing zeros trimmed. Only the invariant-culture rows are ported — the
/// `NumberDecimalSeparator`-varies-by-culture rows are permanently out of scope (this
/// crate has no culture/locale support anywhere, matching `Display`'s existing
/// invariant-only scope for `"c"`).
///
/// Cf. TimeSpanFormat.cs `TryFormatStandard` (`StandardFormat.g` branch),
/// TimeSpanTests.cs#L1593-L1606 (`ToString_TestData`, general short format rows)
#[test]
fn to_string_format_general_short() {
    assert_eq!(
        Ok("142:21:21:18.9101112".to_string()),
        TimeSpan::from_ticks(123_456_789_101_112).to_string_format("g")
    );
    assert_eq!(
        Ok("0:00:00".to_string()),
        TimeSpan::ZERO.to_string_format("g")
    );
    assert_eq!(
        Ok("0:00:00.0000001".to_string()),
        TimeSpan::from_ticks(1).to_string_format("g")
    );
    assert_eq!(
        Ok("-0:00:00.0000001".to_string()),
        TimeSpan::from_ticks(-1).to_string_format("g")
    );
    assert_eq!(
        Ok("10675199:2:48:05.4775807".to_string()),
        TimeSpan::MAX.to_string_format("g")
    );
    assert_eq!(
        Ok("-10675199:2:48:05.4775808".to_string()),
        TimeSpan::MIN.to_string_format("g")
    );
    assert_eq!(
        Ok("1:02:03".to_string()),
        TimeSpan::from_hms(1, 2, 3).unwrap().to_string_format("g")
    );
    assert_eq!(
        Ok("-1:02:03".to_string()),
        (-TimeSpan::from_hms(1, 2, 3).unwrap()).to_string_format("g")
    );
    assert_eq!(
        Ok("12:34:56".to_string()),
        TimeSpan::from_hms(12, 34, 56)
            .unwrap()
            .to_string_format("g")
    );
    assert_eq!(
        Ok("13:10:56:23".to_string()),
        TimeSpan::from_dhms(12, 34, 56, 23)
            .unwrap()
            .to_string_format("g")
    );
    assert_eq!(
        Ok("13:10:56:23.045".to_string()),
        TimeSpan::from_dhms_milli(12, 34, 56, 23, 45)
            .unwrap()
            .to_string_format("g")
    );
    assert_eq!(
        Ok("23:59:59.999".to_string()),
        TimeSpan::from_dhms_milli(0, 23, 59, 59, 999)
            .unwrap()
            .to_string_format("g")
    );
}

/// The general long `"G"` format: always two-digit hours, the day segment always
/// present (`"0:"` when zero), and the fraction always shown at full 7-digit width.
/// Only the invariant-culture rows are ported; see `to_string_format_general_short`'s
/// doc comment for why the culture-varying rows are excluded.
///
/// Cf. TimeSpanFormat.cs `TryFormatStandard` (`StandardFormat.G` branch),
/// TimeSpanTests.cs#L1624-L1636 (`ToString_TestData`, general long format rows)
#[test]
fn to_string_format_general_long() {
    assert_eq!(
        Ok("142:21:21:18.9101112".to_string()),
        TimeSpan::from_ticks(123_456_789_101_112).to_string_format("G")
    );
    assert_eq!(
        Ok("0:00:00:00.0000000".to_string()),
        TimeSpan::ZERO.to_string_format("G")
    );
    assert_eq!(
        Ok("0:00:00:00.0000001".to_string()),
        TimeSpan::from_ticks(1).to_string_format("G")
    );
    assert_eq!(
        Ok("-0:00:00:00.0000001".to_string()),
        TimeSpan::from_ticks(-1).to_string_format("G")
    );
    assert_eq!(
        Ok("10675199:02:48:05.4775807".to_string()),
        TimeSpan::MAX.to_string_format("G")
    );
    assert_eq!(
        Ok("-10675199:02:48:05.4775808".to_string()),
        TimeSpan::MIN.to_string_format("G")
    );
    assert_eq!(
        Ok("0:01:02:03.0000000".to_string()),
        TimeSpan::from_hms(1, 2, 3).unwrap().to_string_format("G")
    );
    assert_eq!(
        Ok("-0:01:02:03.0000000".to_string()),
        (-TimeSpan::from_hms(1, 2, 3).unwrap()).to_string_format("G")
    );
    assert_eq!(
        Ok("0:12:34:56.0000000".to_string()),
        TimeSpan::from_hms(12, 34, 56)
            .unwrap()
            .to_string_format("G")
    );
    assert_eq!(
        Ok("13:10:56:23.0000000".to_string()),
        TimeSpan::from_dhms(12, 34, 56, 23)
            .unwrap()
            .to_string_format("G")
    );
    assert_eq!(
        Ok("13:10:56:23.0450000".to_string()),
        TimeSpan::from_dhms_milli(12, 34, 56, 23, 45)
            .unwrap()
            .to_string_format("G")
    );
    assert_eq!(
        Ok("0:23:59:59.9990000".to_string()),
        TimeSpan::from_dhms_milli(0, 23, 59, 59, 999)
            .unwrap()
            .to_string_format("G")
    );
}

/// The custom-format-string mini-language (`TimeSpanFormat.FormatCustomized`):
/// `%d`/`dd`...`dddddddd` (day), `%h`/`hh` (hour), `%m`/`mm` (minute), `%s`/`ss`
/// (second), `%f`/`ff`...`fffffff` (fraction, truncated, always shown), `%F`/`FF`...
/// `FFFFFFF` (fraction, trailing zeros dropped, omitted if empty), and `\`-escaped
/// literal text.
///
/// Cf. TimeSpanFormat.cs#L296-455 (`FormatCustomized`), TimeSpanTests.cs#L1545-1570
/// (`ToString_TestData`, custom-format rows)
#[test]
fn to_string_format_custom() {
    let input = TimeSpan::from_ticks(123_456_789_101_112);

    let cases: &[(&str, &str)] = &[
        ("%d", "142"),
        ("dd", "142"),
        ("%h", "21"),
        ("hh", "21"),
        ("%m", "21"),
        ("mm", "21"),
        ("%s", "18"),
        ("ss", "18"),
        ("%f", "9"),
        ("ff", "91"),
        ("fff", "910"),
        ("ffff", "9101"),
        ("fffff", "91011"),
        ("ffffff", "910111"),
        ("fffffff", "9101112"),
        ("%F", "9"),
        ("FF", "91"),
        ("FFF", "91"),
        ("FFFF", "9101"),
        ("FFFFF", "91011"),
        ("FFFFFF", "910111"),
        ("FFFFFFF", "9101112"),
        ("dd\\.ss", "142.18"),
        ("dddddd\\.ss", "000142.18"),
    ];

    for (format, expected) in cases {
        assert_eq!(
            Ok((*expected).to_string()),
            input.to_string_format(format),
            "format {format:?}"
        );
    }
}

/// Quoted literal spans (`'...'`/`"..."`) are copied verbatim into the output,
/// including `\`-escaped characters within the quotes.
///
/// Cf. TimeSpanFormat.cs#L405-408 (`FormatCustomized`'s `'\''`/`'"'` case),
/// DateTimeFormat.cs#L284-337 (`ParseQuoteString`)
#[test]
fn to_string_format_custom_quoted_literal() {
    let ts = TimeSpan::from_hms(1, 2, 3).unwrap();
    assert_eq!(
        Ok("hh is 01".to_string()),
        ts.to_string_format("'hh is 'hh")
    );
    assert_eq!(
        Ok("hh is 01".to_string()),
        ts.to_string_format("\"hh is \"hh")
    );
    // A backslash-escaped character inside a quoted span is unescaped into the
    // literal output, per `DateTimeFormat.ParseQuoteString`'s own `\`-handling.
    assert_eq!(
        Ok("it's 01".to_string()),
        ts.to_string_format("'it\\'s 'hh")
    );
}

/// `FormatCustomized` never writes a sign character itself — unlike the standard
/// `"c"`/`"g"`/`"G"` formats (which all prepend `-` for a negative `TimeSpan`), a
/// custom format string has no specifier for the sign, so a negative `TimeSpan`
/// formats identically to its positive magnitude. This is a genuine upstream quirk
/// (no case in `FormatCustomized`'s switch ever emits `-`), not a bug this port
/// introduces.
///
/// Cf. TimeSpanFormat.cs#L301-312 (`day`/`time` are negated to non-negative
/// magnitudes before the tokenizer loop runs; no `-` is ever appended)
#[test]
fn to_string_format_custom_no_sign() {
    let positive = TimeSpan::from_hms(1, 2, 3).unwrap();
    let negative = -positive;
    assert_eq!(
        Ok("01:02:03".to_string()),
        positive.to_string_format("hh\\:mm\\:ss")
    );
    assert_eq!(
        Ok("01:02:03".to_string()),
        negative.to_string_format("hh\\:mm\\:ss")
    );
}

/// A single-character format outside `"c"`/`"t"`/`"T"`/`"g"`/`"G"` is always rejected
/// at the top level (C#'s `Format`/`TryFormat` special-case format strings of length 1
/// entirely separately from the custom-format tokenizer, even though some of those
/// single characters, e.g. `"d"`, would otherwise be valid custom-format tokens). A
/// syntactically-invalid custom format string (length != 1) reports
/// [`TimeSpanError::InvalidFormat`] rather than panicking, mirroring C#'s
/// `FormatException`.
///
/// Cf. TimeSpanFormat.cs#L26-41 (`Format`'s length-1 special case, checked before
/// ever reaching `FormatCustomized`), TimeSpanTests.cs#L1671-L1684
/// (`ToString_InvalidFormat_TestData`, `ToString_InvalidFormat_ThrowsFormatException`)
#[test]
fn to_string_format_invalid() {
    // TimeSpanTests.cs#L1673-L1676: single characters that aren't valid standard
    // format specifiers (uppercase "C" is deliberately invalid in C# too - only
    // lowercase "c" is the constant format; "F"/"d" are custom-format-only tokens,
    // never reachable as a length-1 format string).
    for format in ["y", "F", "C", "d"] {
        assert_eq!(
            Err(TimeSpanError::InvalidFormat),
            TimeSpan::ZERO.to_string_format(format)
        );
    }
    // TimeSpanTests.cs#L1674: "cc" is a 2-character custom format string in C# -
    // invalid there too, since 'c' isn't a recognized custom-format token.
    for format in [
        "cc",            // 'c' isn't a recognized custom-format token
        "hhh",           // 'h' run > 2 (TimeSpanFormat.cs#L326-329)
        "mmm",           // 'm' run > 2 (TimeSpanFormat.cs#L334-337)
        "sss",           // 's' run > 2 (TimeSpanFormat.cs#L342-345)
        "ffffffff",      // 'f' run > 7 (TimeSpanFormat.cs#L353-356)
        "FFFFFFFF",      // 'F' run > 7 (TimeSpanFormat.cs#L367-370)
        "ddddddddd",     // 'd' run > 8 (TimeSpanFormat.cs#L398-401)
        "'unterminated", // missing closing quote (DateTimeFormat.cs#L327-331)
        "'bad\\",        // '\' at the end of a quoted span (DateTimeFormat.cs#L309-319)
        "dd%",           // trailing '%' (TimeSpanFormat.cs#L416-429)
        "dd%%",          // "%%" is disallowed (TimeSpanFormat.cs#L416-429)
        "dd\\",          // trailing '\' (TimeSpanFormat.cs#L436-447)
        "dXd",           // unquoted/unescaped literal character (TimeSpanFormat.cs#L449-451)
    ] {
        assert_eq!(
            Err(TimeSpanError::InvalidFormat),
            TimeSpan::ZERO.to_string_format(format),
            "format {format:?}"
        );
    }
}

/// The non-allocating counterpart to `to_string_format`: writes UTF-8 bytes directly
/// into a caller-provided buffer instead of allocating a `String`. Mirrors C#'s
/// `TryFormat(Span<char>, out int charsWritten, ...)`/`TryFormat(Span<byte>, out int
/// bytesWritten, ...)` buffer-sizing behavior — a buffer exactly one byte too short
/// reports [`TimeSpanError::InsufficientBuffer`] and writes nothing, a buffer exactly
/// long enough succeeds and is filled completely, and a buffer one byte larger than
/// needed succeeds while leaving the trailing byte untouched.
///
/// Cf. TimeSpanTests.cs#L1843-L1888 (`TryFormat_Valid`, `ToString_TestData` rows)
#[test]
fn try_format_valid() {
    let cases: &[(TimeSpan, &str, &str)] = &[
        (TimeSpan::from_hms(1, 2, 3).unwrap(), "c", "01:02:03"),
        (TimeSpan::ZERO, "c", "00:00:00"),
        (
            TimeSpan::from_ticks(123_456_789_101_112),
            "c",
            "142.21:21:18.9101112",
        ),
        (TimeSpan::MIN, "c", "-10675199.02:48:05.4775808"),
        (TimeSpan::from_hms(1, 2, 3).unwrap(), "g", "1:02:03"),
        (TimeSpan::ZERO, "g", "0:00:00"),
        (TimeSpan::MIN, "g", "-10675199:2:48:05.4775808"),
        (
            TimeSpan::from_hms(1, 2, 3).unwrap(),
            "G",
            "0:01:02:03.0000000",
        ),
        (TimeSpan::ZERO, "G", "0:00:00:00.0000000"),
        (TimeSpan::MIN, "G", "-10675199:02:48:05.4775808"),
    ];

    for (input, format, expected) in cases {
        let expected_len = expected.len();

        // One byte too short: fails, and nothing is written.
        let mut too_small = vec![0u8; expected_len - 1];
        assert_eq!(
            Err(TimeSpanError::InsufficientBuffer),
            input.try_format(&mut too_small, format)
        );

        // Exactly long enough: succeeds, buffer filled exactly.
        let mut exact = vec![0u8; expected_len];
        let written = input.try_format(&mut exact, format).unwrap();
        assert_eq!(expected_len, written);
        assert_eq!(*expected, std::str::from_utf8(&exact).unwrap());

        // One byte larger than needed: succeeds, trailing byte left untouched.
        let mut larger = vec![0u8; expected_len + 1];
        let written = input.try_format(&mut larger, format).unwrap();
        assert_eq!(expected_len, written);
        assert_eq!(*expected, std::str::from_utf8(&larger[..written]).unwrap());
        assert_eq!(0, larger[larger.len() - 1]);
    }
}

/// The custom-format-string counterpart to `try_format_valid`: `try_format`'s
/// non-allocating path also covers the custom-format-string mini-language, not just
/// the five standard formats — mirroring `to_string_format_custom`'s cases and
/// `try_format_valid`'s buffer-sizing contract (one byte short fails and writes
/// nothing, exact-size succeeds and fills completely, one byte larger succeeds and
/// leaves the trailing byte untouched).
///
/// Cf. TimeSpanTests.cs#L1843-L1888 (`TryFormat_Valid`, sharing `ToString_TestData`
/// with `ToString(string)` — TimeSpanTests.cs#L1546-1570's custom-format rows)
#[test]
fn try_format_custom() {
    let input = TimeSpan::from_ticks(123_456_789_101_112);

    let cases: &[(&str, &str)] = &[
        ("%d", "142"),
        ("dd", "142"),
        ("%h", "21"),
        ("hh", "21"),
        ("%m", "21"),
        ("mm", "21"),
        ("%s", "18"),
        ("ss", "18"),
        ("%f", "9"),
        ("fffffff", "9101112"),
        ("%F", "9"),
        ("FFFFFFF", "9101112"),
        ("dd\\.ss", "142.18"),
        ("dddddd\\.ss", "000142.18"),
    ];

    for (format, expected) in cases {
        let expected_len = expected.len();

        // One byte too short: fails, and nothing is written.
        let mut too_small = vec![0u8; expected_len - 1];
        assert_eq!(
            Err(TimeSpanError::InsufficientBuffer),
            input.try_format(&mut too_small, format),
            "format {format:?}"
        );

        // Exactly long enough: succeeds, buffer filled exactly.
        let mut exact = vec![0u8; expected_len];
        let written = input.try_format(&mut exact, format).unwrap();
        assert_eq!(expected_len, written, "format {format:?}");
        assert_eq!(
            *expected,
            std::str::from_utf8(&exact).unwrap(),
            "format {format:?}"
        );

        // One byte larger than needed: succeeds, trailing byte left untouched.
        let mut larger = vec![0u8; expected_len + 1];
        let written = input.try_format(&mut larger, format).unwrap();
        assert_eq!(expected_len, written, "format {format:?}");
        assert_eq!(
            *expected,
            std::str::from_utf8(&larger[..written]).unwrap(),
            "format {format:?}"
        );
        assert_eq!(0, larger[larger.len() - 1], "format {format:?}");
    }
}

/// Custom format strings never write a sign character, matching
/// `to_string_format_custom_no_sign` — verified here through the non-allocating
/// `try_format` path too.
///
/// Cf. TimeSpanFormat.cs#L301-312
#[test]
fn try_format_custom_no_sign() {
    let positive = TimeSpan::from_hms(1, 2, 3).unwrap();
    let negative = -positive;

    let mut buf = [0u8; 8];
    let written = positive.try_format(&mut buf, "hh\\:mm\\:ss").unwrap();
    assert_eq!("01:02:03", std::str::from_utf8(&buf[..written]).unwrap());

    let mut buf = [0u8; 8];
    let written = negative.try_format(&mut buf, "hh\\:mm\\:ss").unwrap();
    assert_eq!("01:02:03", std::str::from_utf8(&buf[..written]).unwrap());
}

/// Mirrors `to_string_format_invalid`, but through `try_format`: an invalid format
/// string reports [`TimeSpanError::InvalidFormat`] regardless of buffer size, checked
/// before any buffer-length validation (matching C#, where `FormatException` is thrown
/// even when passed a 1-element destination span) — including a syntactically invalid
/// *custom* format string, not just an invalid single-character standard format.
///
/// Cf. TimeSpanTests.cs#L1890-L1896 (`TryFormat_InvalidFormat_ThrowsFormatException`)
#[test]
fn try_format_invalid_format() {
    for format in ["y", "F", "C", "cc"] {
        let mut buf = [0u8; 1];
        assert_eq!(
            Err(TimeSpanError::InvalidFormat),
            TimeSpan::ZERO.try_format(&mut buf, format)
        );
    }

    // "hhh": an 'h' run > 2 is a syntactically invalid *custom* format string
    // (TimeSpanFormat.cs#L326-329) — rejected the same way as an invalid standard
    // format, even though it's a multi-character string that reaches the custom-
    // format tokenizer rather than the standard-format special case.
    let mut buf = [0u8; 1];
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::from_ticks(123_456_789_101_112).try_format(&mut buf, "hhh")
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
