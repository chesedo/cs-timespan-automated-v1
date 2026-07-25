use std::fmt;

/// Failure modes for [`crate::TimeSpan`] construction, arithmetic, and parsing.
///
/// Clusters several distinct C# exception types that share a Rust-relevant cause:
///
/// - [`TimeSpanError::Overflow`] covers both `ArgumentOutOfRangeException` (raised by
///   constructors and integer `FromX` overloads when a component is out of range) and
///   `OverflowException` (raised by arithmetic, `Duration`, `Multiply`, `Divide`, and
///   double `FromX` overloads) — in both cases the result can't be represented as a
///   valid `TimeSpan`.
/// - [`TimeSpanError::NotANumber`] covers `ArgumentException` raised when a `f64`
///   argument is NaN.
/// - [`TimeSpanError::InvalidFormat`] covers `FormatException` from parsing and from
///   [`crate::TimeSpan::to_string_format`]'s format-string validation; kept as a bare
///   variant until a `work-issue` implementing more of `Parse`/`ParseExact`/
///   `ToString(format)` needs to say more about what was invalid.
/// - [`TimeSpanError::InsufficientBuffer`] is new relative to C#: C#'s
///   `TryFormat(Span<TChar>, out int charsWritten, ...)` reports a destination buffer
///   too short to hold the output via a `bool` return (`false`) rather than an
///   exception. This crate's other fallible methods all return `Result`, so
///   [`crate::TimeSpan::try_format`] keeps that pattern instead of adding a
///   `bool`-returning method that would be the only one of its kind in the crate; this
///   variant is what "too short" maps to. It's kept distinct from
///   [`TimeSpanError::InvalidFormat`] since the two are unrelated failure causes (a bad
///   format string vs. a correctly-formatted result that doesn't fit).
///
/// Two C# failure modes have no Rust equivalent at all: `CompareTo(object)`'s
/// "must be TimeSpan" `ArgumentException` (Rust has no dynamically-typed `object`
/// overload to port — `Ord`/`PartialOrd` replace it), and `TimeSpanStyles` validation
/// (an out-of-range enum value is unrepresentable in Rust's type system in the first
/// place).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSpanError {
    /// The result falls outside the range representable by `TimeSpan`.
    Overflow,
    /// A floating-point argument was NaN where a finite value was required.
    NotANumber,
    /// The input could not be parsed as a `TimeSpan`.
    InvalidFormat,
    /// The destination buffer passed to [`crate::TimeSpan::try_format`] is too short
    /// to hold the formatted output.
    InsufficientBuffer,
}

impl fmt::Display for TimeSpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeSpanError::Overflow => write!(f, "TimeSpan overflowed its representable range"),
            TimeSpanError::NotANumber => write!(f, "value must not be NaN"),
            TimeSpanError::InvalidFormat => write!(f, "invalid TimeSpan format"),
            TimeSpanError::InsufficientBuffer => {
                write!(
                    f,
                    "destination buffer is too short for the formatted TimeSpan"
                )
            }
        }
    }
}

impl std::error::Error for TimeSpanError {}
