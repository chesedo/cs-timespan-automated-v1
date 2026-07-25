//! Legacy `"c"`/`"t"`/`"T"` constant-format `TimeSpan::parse_exact` fast path.
//!
//! Ports `TimeSpanParse.cs`'s `TryParseTimeSpanConstant`/`StringParser`
//! (TimeSpanParse.cs#L1462-1659) — described upstream as "100% identical to the
//! non-globalized v1.0-v3.5 `TimeSpan.Parse()` routine", kept for legacy/perf reasons. All
//! three of `"c"`, `"t"`, `"T"` dispatch to this exact same algorithm upstream
//! (`TryParseExactTimeSpan`'s `switch`, TimeSpanParse.cs#L1237-1239, never inspects which of
//! the three characters selected it).
//!
//! This is a fixed-position parser, genuinely distinct from both `time_span_parse.rs`'s
//! token/candidate-based standard-format parser and `time_span_parse_exact.rs`'s
//! custom-format-string tokenizer: it walks the *input* directly, char by char, matching a
//! hardcoded `[-]d[.]hh:mm[:ss[.fffffff]]` shape (day segment, and everything after it,
//! optional) rather than tokenizing into a number/separator stream and pattern-matching the
//! resulting shape against a literal table.

use crate::{TimeSpan, TimeSpanError};

/// Cf. `StringParser` (TimeSpanParse.cs#L1469-1659) — a `ref struct` walking `_str` via
/// `_pos`/`_ch` rather than an iterator, mirrored here the same way.
struct StringParser {
    chars: Vec<char>,
    /// Index of the current character (`ch`). Starts at `-1` (mirroring `_pos = -1`,
    /// TimeSpanParse.cs#L1494) so the first `next_char` call lands on index `0`.
    pos: i64,
    ch: char,
}

impl StringParser {
    fn new(input: &str) -> Self {
        let mut parser = StringParser {
            chars: input.chars().collect(),
            pos: -1,
            ch: '\0',
        };
        parser.next_char();
        parser
    }

    /// Cf. StringParser.NextChar (TimeSpanParse.cs#L1475-1487): returns `'\0'` once past the
    /// end (the sentinel every call site below relies on, same as `CharTokenizer::next_char`
    /// in `time_span_parse_exact.rs`), and stops advancing `pos` once at the end too.
    fn next_char(&mut self) {
        if self.pos < self.chars.len() as i64 {
            self.pos += 1;
        }
        let idx = self.pos;
        self.ch = if idx >= 0 && (idx as usize) < self.chars.len() {
            self.chars[idx as usize]
        } else {
            '\0'
        };
    }

    /// Looks ahead (without consuming) for the first non-digit character from the current
    /// position onward, or `'\0'` if none remains.
    ///
    /// Cf. StringParser.NextNonDigit (TimeSpanParse.cs#L1489-1493)
    fn next_non_digit(&self) -> char {
        let start = if self.pos < 0 { 0 } else { self.pos as usize }.min(self.chars.len());
        self.chars[start..]
            .iter()
            .copied()
            .find(|c| !c.is_ascii_digit())
            .unwrap_or('\0')
    }

    /// Cf. StringParser.SkipBlanks (TimeSpanParse.cs#L1656-1659) — only ASCII space/tab, not
    /// general Unicode whitespace (unlike `time_span_parse.rs`'s `str::trim`).
    fn skip_blanks(&mut self) {
        while self.ch == ' ' || self.ch == '\t' {
            self.next_char();
        }
    }

    /// Cf. StringParser.ParseInt (TimeSpanParse.cs#L1567-1589). The C# version guards its
    /// `int i` accumulator against 32-bit overflow with `(i & 0xF0000000) != 0` before every
    /// multiply; kept here even though the `i64` accumulator can't itself overflow for any
    /// `max` this module passes in, to stay a faithful line-for-line port rather than a
    /// "fixed" one — and it still serves its original purpose of rejecting a pathologically
    /// long digit run (e.g. fifty digits) before it grows unboundedly.
    fn parse_int(&mut self, max: i64) -> Result<i64, TimeSpanError> {
        let mut value: i64 = 0;
        let start_pos = self.pos;

        while self.ch.is_ascii_digit() {
            if (value & 0xF000_0000) != 0 {
                return Err(TimeSpanError::Overflow);
            }
            value = value * 10 + self.ch.to_digit(10).unwrap() as i64;
            self.next_char();
        }

        if start_pos == self.pos {
            return Err(TimeSpanError::InvalidFormat);
        }
        if value > max {
            return Err(TimeSpanError::Overflow);
        }
        Ok(value)
    }

    /// Parses `hh:mm[:ss[.fffffff]]`, returning the resulting tick count.
    ///
    /// Cf. StringParser.ParseTime (TimeSpanParse.cs#L1591-1627)
    fn parse_time(&mut self) -> Result<i64, TimeSpanError> {
        let hours = self.parse_int(23)?;
        let mut time = hours.wrapping_mul(TimeSpan::TICKS_PER_HOUR);

        if self.ch != ':' {
            return Err(TimeSpanError::InvalidFormat);
        }
        self.next_char();

        let minutes = self.parse_int(59)?;
        time = time.wrapping_add(minutes.wrapping_mul(TimeSpan::TICKS_PER_MINUTE));

        if self.ch == ':' {
            self.next_char();

            // Cf. TimeSpanParse.cs#L1611-1616: seconds are only parsed when not immediately
            // followed by the fraction separator, allowing "hh:mm:.f" to elide them.
            if self.ch != '.' {
                let seconds = self.parse_int(59)?;
                time = time.wrapping_add(seconds.wrapping_mul(TimeSpan::TICKS_PER_SECOND));
            }

            if self.ch == '.' {
                self.next_char();
                let mut fraction_place = TimeSpan::TICKS_PER_SECOND;
                while fraction_place > 1 && self.ch.is_ascii_digit() {
                    fraction_place /= 10;
                    let digit = self.ch.to_digit(10).unwrap() as i64;
                    time = time.wrapping_add(digit.wrapping_mul(fraction_place));
                    self.next_char();
                }
            }
        }

        Ok(time)
    }
}

/// Parses `input` per the legacy `"c"`/`"t"`/`"T"` constant-format algorithm.
///
/// Cf. TryParseTimeSpanConstant/StringParser.TryParse (TimeSpanParse.cs#L1466-1467,
/// #L1497-1531)
pub(crate) fn parse_constant(input: &str) -> Result<TimeSpan, TimeSpanError> {
    let mut parser = StringParser::new(input);
    parser.skip_blanks();

    let negative = if parser.ch == '-' {
        parser.next_char();
        true
    } else {
        false
    };

    // Cf. TimeSpanParse.cs#L1517-1518: `(int)(0x7FFFFFFFFFFFFFFFL / TimeSpan.TicksPerDay)`.
    let max_days = i64::MAX / TimeSpan::TICKS_PER_DAY;

    let mut time = if parser.next_non_digit() == ':' {
        parser.parse_time()?
    } else {
        let days = parser.parse_int(max_days)?;
        let mut time = days.wrapping_mul(TimeSpan::TICKS_PER_DAY);
        if parser.ch == '.' {
            parser.next_char();
            time = time.wrapping_add(parser.parse_time()?);
        }
        time
    };

    if negative {
        time = time.wrapping_neg();
        if time > 0 {
            return Err(TimeSpanError::Overflow);
        }
    } else if time < 0 {
        return Err(TimeSpanError::Overflow);
    }

    parser.skip_blanks();
    if (parser.pos as usize) < parser.chars.len() {
        return Err(TimeSpanError::InvalidFormat);
    }

    Ok(TimeSpan::from_ticks(time))
}
