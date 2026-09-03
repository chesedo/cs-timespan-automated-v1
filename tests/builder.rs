//! Tests for `TimeSpan::builder()` / `TimeSpanBuilder` — the fluent, Rust-idiomatic
//! alternative to the DHMS constructor family and the `*_parts` factories.

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError};

/// A builder with every field set should match the expected tick count for
/// 1d 2h 3m 4s 5ms 6μs.
#[test]
fn basic_construction_all_fields() {
    let built = TimeSpan::builder()
        .days(1)
        .hours(2)
        .minutes(3)
        .seconds(4)
        .milliseconds(5)
        .microseconds(6)
        .build();

    assert_eq!(built, Ok(TimeSpan::from_ticks(937_840_050_060)));
}

/// Setting only a subset of fields leaves the rest at their `0` default.
#[test]
fn basic_construction_partial_fields() {
    let built = TimeSpan::builder().days(1).hours(2).minutes(30).build();

    assert_eq!(built, Ok(TimeSpan::from_ticks(954_000_000_000)));
}

/// Setters are fluent and order-independent (each just assigns its own field).
#[test]
fn setters_are_order_independent() {
    let a = TimeSpan::builder()
        .days(1)
        .hours(2)
        .minutes(3)
        .seconds(4)
        .build();
    let b = TimeSpan::builder()
        .seconds(4)
        .minutes(3)
        .hours(2)
        .days(1)
        .build();

    assert_eq!(a, b);
}

/// Calling a setter more than once keeps only the last value.
#[test]
fn setter_called_twice_keeps_last_value() {
    let built = TimeSpan::builder().days(1).days(2).build();

    assert_eq!(built, Ok(TimeSpan::from_ticks(1_728_000_000_000)));
}

/// `build()` with no fields set at all is equivalent to `TimeSpan::ZERO`.
#[test]
fn zero_default() {
    assert_eq!(Ok(TimeSpan::ZERO), TimeSpan::builder().build());
}

/// Negative components are supported, matching the DHMS family's own sign handling.
#[test]
fn negative_components() {
    let built = TimeSpan::builder().days(-1).hours(-2).build();

    assert_eq!(built, Ok(TimeSpan::from_ticks(-936_000_000_000)));
}

/// A combined value overflowing the representable tick range is reported as
/// `TimeSpanError::Overflow`, matching `dhms_to_ticks`/the `_parts` family.
#[test]
fn overflow_detection() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::builder().days(i64::MAX).build()
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::builder().days(i64::MIN).build()
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::builder()
            .days(TimeSpan::MAX.ticks() / TimeSpan::TICKS_PER_DAY + 1)
            .build()
    );
}

/// A value that overflows on one field alone still overflows once combined with
/// other in-range fields (the sum, not any single field, is what's bounds-checked).
#[test]
fn overflow_detection_combined_fields() {
    let max_days = TimeSpan::MAX.ticks() / TimeSpan::TICKS_PER_DAY;

    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::builder().days(max_days).hours(24).build()
    );
}

/// At the exact representable boundary (the largest whole microsecond count that
/// still fits), `build()` succeeds with the expected tick count; one microsecond
/// further overflows.
#[test]
fn boundary_values_succeed() {
    let max_microseconds = i64::MAX / TimeSpan::TICKS_PER_MICROSECOND;

    assert_eq!(
        Ok(TimeSpan::from_ticks(
            max_microseconds * TimeSpan::TICKS_PER_MICROSECOND
        )),
        TimeSpan::builder().microseconds(max_microseconds).build()
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::builder()
            .microseconds(max_microseconds + 1)
            .build()
    );
}
