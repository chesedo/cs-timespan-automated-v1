//! Tests for `TimeSpan` arithmetic: the `std::ops` impls and the
//! `checked_*`/`duration`/`divide_time_span` methods.

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError};

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
#[should_panic(expected = "TimeSpan addition overflowed its representable range")]
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
#[should_panic(expected = "TimeSpan subtraction overflowed its representable range")]
fn sub_operator_overflow_panics() {
    let _ = TimeSpan::MIN - TimeSpan::from_ticks(1);
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

/// Ported from `MultiplicationTestData`, shared by both C#'s `Multiplication` and
/// `Division` theories (the latter derives its divisor as `1.0 / factor`). Exercised
/// here against `checked_mul`, `Mul<f64> for TimeSpan`, and `Mul<TimeSpan> for f64`.
///
/// Cf. TimeSpanTests.cs#L1718-L1728 (`MultiplicationTestData`), TimeSpanTests.cs#L1754-L1759
/// (`Multiplication`)
fn multiplication_test_data() -> [(TimeSpan, f64, TimeSpan); 8] {
    [
        (
            TimeSpan::builder().hours(2).minutes(30).build().unwrap(),
            2.0,
            TimeSpan::builder().hours(5).build().unwrap(),
        ),
        (
            TimeSpan::builder()
                .days(14)
                .hours(2)
                .minutes(30)
                .build()
                .unwrap(),
            192.0,
            TimeSpan::from_days_i32(2708).unwrap(),
        ),
        (
            TimeSpan::from_days(366.0).unwrap(),
            std::f64::consts::PI,
            TimeSpan::from_ticks(993_446_995_288_779),
        ),
        (
            TimeSpan::from_days(366.0).unwrap(),
            -std::f64::consts::E,
            TimeSpan::from_ticks(-859_585_952_922_633),
        ),
        (
            TimeSpan::from_days(29.530587981).unwrap(),
            13.0,
            TimeSpan::from_days(29.530587981 * 13.0).unwrap(),
        ),
        (
            TimeSpan::from_days(-29.530587981).unwrap(),
            -12.0,
            TimeSpan::from_days(-29.530587981 * -12.0).unwrap(),
        ),
        (
            TimeSpan::from_days(-29.530587981).unwrap(),
            0.0,
            TimeSpan::ZERO,
        ),
        (
            TimeSpan::MAX,
            0.5,
            TimeSpan::from_ticks((i64::MAX as f64 * 0.5) as i64),
        ),
    ]
}

/// Cf. TimeSpanTests.cs#L1754-L1759 (`Multiplication`)
#[test]
fn checked_mul_basic() {
    for (time_span, factor, expected) in multiplication_test_data() {
        assert_eq!(Ok(expected), time_span.checked_mul(factor));
        assert_eq!(expected, time_span * factor);
        assert_eq!(expected, factor * time_span);
    }
}

/// `TimeSpan.MaxValue * 1.000000001` throws `OverflowException` in C#, and the
/// reversed operand order (`-1.000000001 * TimeSpan.MaxValue`) does too, since it
/// delegates to the same `operator*(TimeSpan, double)` (TimeSpan.cs#L922).
///
/// Cf. TimeSpanTests.cs#L1761-L1766 (`OverflowingMultiplication`)
#[test]
fn checked_mul_overflow() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::MAX.checked_mul(1.000000001)
    );
}

/// `operator*` panics rather than returning a `Result`, mirroring `checked_add`'s
/// established overflow-panics-at-the-operator-layer pattern.
///
/// Cf. TimeSpanTests.cs#L1761-L1766 (`OverflowingMultiplication`)
#[test]
#[should_panic(
    expected = "TimeSpan multiplication overflowed its representable range, or factor was NaN"
)]
fn mul_operator_overflow_panics() {
    let _ = TimeSpan::MAX * 1.000000001;
}

/// Same as `mul_operator_overflow_panics`, but through the reversed
/// `f64 * TimeSpan` operand order.
///
/// Cf. TimeSpanTests.cs#L1761-L1766 (`OverflowingMultiplication`)
#[test]
#[should_panic(
    expected = "TimeSpan multiplication overflowed its representable range, or factor was NaN"
)]
fn mul_operator_reversed_overflow_panics() {
    let _ = -1.000000001 * TimeSpan::MAX;
}

/// Cf. TimeSpanTests.cs#L1768-L1773 (`NaNMultiplication`)
#[test]
fn checked_mul_nan() {
    assert_eq!(
        Err(TimeSpanError::NotANumber),
        TimeSpan::from_days(1.0).unwrap().checked_mul(f64::NAN)
    );
}

/// Cf. TimeSpanTests.cs#L1768-L1773 (`NaNMultiplication`)
#[test]
#[should_panic(
    expected = "TimeSpan multiplication overflowed its representable range, or factor was NaN"
)]
fn mul_operator_nan_panics() {
    let _ = TimeSpan::from_days(1.0).unwrap() * f64::NAN;
}

/// Cf. TimeSpanTests.cs#L1768-L1773 (`NaNMultiplication`)
#[test]
#[should_panic(
    expected = "TimeSpan multiplication overflowed its representable range, or factor was NaN"
)]
fn mul_operator_reversed_nan_panics() {
    let _ = f64::NAN * TimeSpan::from_days(1.0).unwrap();
}

/// C#'s `Math.Round(double)` (no `MidpointRounding` argument) uses banker's rounding
/// (round-half-to-even), the opposite of Rust's `f64::round()` (round-half-away-from-
/// zero). `5 ticks * 0.5` raw-multiplies to exactly `2.5`, an exact `.5`-tick
/// midpoint: `f64::round()` would give `3.0`, but C# — and this method — must give
/// `2.0` (the nearest even integer). This is the specific divergence issue #51 was
/// filed to close.
///
/// Cf. TimeSpan.cs#L915-L917 (`Math.Round(timeSpan.Ticks * factor)`)
#[test]
fn checked_mul_rounds_half_to_even() {
    // 5 * 0.5 == 2.5, exact midpoint: rounds down to 2 (even), not up to 3.
    assert_eq!(
        Ok(TimeSpan::from_ticks(2)),
        TimeSpan::from_ticks(5).checked_mul(0.5)
    );
    // 3 * 0.5 == 1.5, exact midpoint: rounds up to 2 (even), not down to 1.
    assert_eq!(
        Ok(TimeSpan::from_ticks(2)),
        TimeSpan::from_ticks(3).checked_mul(0.5)
    );
    // The negative mirror of the first case: -5 * 0.5 == -2.5, rounds to -2 (even),
    // not -3, which `f64::round()`'s away-from-zero behavior would give.
    assert_eq!(
        Ok(TimeSpan::from_ticks(-2)),
        TimeSpan::from_ticks(-5).checked_mul(0.5)
    );
}

/// Ported from C#'s `Division` theory, which reuses `MultiplicationTestData` by
/// dividing by `1.0 / factor` and asserting the same `expected` result — i.e.
/// `timeSpan / (1.0 / factor) == timeSpan * factor`. Exercised here against
/// `checked_div` and `Div<f64> for TimeSpan`.
///
/// Cf. TimeSpanTests.cs#L1775-L1781 (`Division`)
#[test]
fn checked_div_basic() {
    for (time_span, factor, expected) in multiplication_test_data() {
        let divisor = 1.0 / factor;
        assert_eq!(Ok(expected), time_span.checked_div(divisor));
        assert_eq!(expected, time_span / divisor);
    }
}

/// Only the `TimeSpan / double` half of C#'s `DivideByZero` test; the
/// `TimeSpan / TimeSpan` half (dividing by `TimeSpan::ZERO`) is already covered by
/// `divide_time_span_operator_by_zero_yields_infinity` and
/// `divide_time_span_operator_zero_by_zero_yields_nan`.
///
/// Cf. TimeSpanTests.cs#L1783-L1792 (`DivideByZero`)
#[test]
fn checked_div_by_zero() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days(1.0).unwrap().checked_div(0.0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::from_days(-1.0).unwrap().checked_div(0.0)
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::ZERO.checked_div(0.0)
    );
}

/// Cf. TimeSpanTests.cs#L1783-L1792 (`DivideByZero`)
#[test]
#[should_panic(
    expected = "TimeSpan division overflowed its representable range, or divisor was NaN"
)]
fn div_operator_by_zero_panics() {
    let _ = TimeSpan::from_days(1.0).unwrap() / 0.0;
}

/// Cf. TimeSpanTests.cs#L1794-L1798 (`NaNDivision`)
#[test]
fn checked_div_nan() {
    assert_eq!(
        Err(TimeSpanError::NotANumber),
        TimeSpan::from_days(1.0).unwrap().checked_div(f64::NAN)
    );
}

/// Cf. TimeSpanTests.cs#L1794-L1798 (`NaNDivision`)
#[test]
#[should_panic(
    expected = "TimeSpan division overflowed its representable range, or divisor was NaN"
)]
fn div_operator_nan_panics() {
    let _ = TimeSpan::from_days(1.0).unwrap() / f64::NAN;
}

/// Same round-half-to-even requirement as `checked_mul_rounds_half_to_even`, but via
/// `checked_div`.
///
/// Cf. TimeSpan.cs#L932 (`Math.Round(timeSpan.Ticks / divisor)`)
#[test]
fn checked_div_rounds_half_to_even() {
    // 5 / 2.0 == 2.5, exact midpoint: rounds down to 2 (even).
    assert_eq!(
        Ok(TimeSpan::from_ticks(2)),
        TimeSpan::from_ticks(5).checked_div(2.0)
    );
    // 3 / 2.0 == 1.5, exact midpoint: rounds up to 2 (even).
    assert_eq!(
        Ok(TimeSpan::from_ticks(2)),
        TimeSpan::from_ticks(3).checked_div(2.0)
    );
}
