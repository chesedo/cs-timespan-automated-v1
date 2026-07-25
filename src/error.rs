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
}

impl fmt::Display for TimeSpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeSpanError::Overflow => write!(f, "TimeSpan overflowed its representable range"),
            TimeSpanError::NotANumber => write!(f, "value must not be NaN"),
            TimeSpanError::InvalidFormat => write!(f, "invalid TimeSpan format"),
        }
    }
}

impl std::error::Error for TimeSpanError {}
