//! Gungraun (instruction-count) benchmark for `TimeSpan::parse_exact`/`parse_exact_multiple`,
//! the format-string-driven parsing path — structurally distinct from `parse_gungraun.rs`'s
//! `Parse`/`from_str` coverage (see issue #73 and `src/time_span_parse_exact.rs`'s doc comment
//! for the tokenizer design).
//!
//! `#[bench::single_format]` exercises digit-matching, escape-handling, and fraction-matching
//! together in one format string. `#[bench::multiple_formats]` covers `parse_exact_multiple`'s
//! retry-until-match cost: two candidate formats fail before the third matches `INPUT`.
use std::hint::black_box;

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError, TimeSpanStyles};
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};

/// `"01.02:03:04.5000000"` — a full days.hours:minutes:seconds.fraction value, chosen so a
/// single input drives every branch the format strings below exercise.
const INPUT: &str = "01.02:03:04.5000000";

/// Digit-matching (`dd`/`hh`/`mm`/`ss`), escape-handling (`\.`/`\:`), and fraction-matching
/// (`fffffff`) all in one format string.
const FORMAT: &str = r"dd\.hh\:mm\:ss\.fffffff";

#[library_benchmark]
#[bench::single_format(FORMAT)]
fn bench_parse_exact(format: &str) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact(
        black_box(INPUT),
        black_box(format),
        TimeSpanStyles::None,
    ))
}

/// Three candidate formats, only the last of which matches `INPUT` — `parse_exact_multiple`
/// must fail the first two attempts before succeeding on the third.
const FORMATS: [&str; 3] = [r"hh\:mm\:ss", "%h", FORMAT];

#[library_benchmark]
#[bench::multiple_formats(&FORMATS)]
fn bench_parse_exact_multiple(formats: &[&str]) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact_multiple(
        black_box(INPUT),
        black_box(formats),
        TimeSpanStyles::None,
    ))
}

library_benchmark_group!(
    name = parse_exact_group,
    benchmarks = [bench_parse_exact, bench_parse_exact_multiple]
);

main!(
    // 5% is comfortably above callgrind's own binary-layout noise floor (a few tenths of a
    // percent) while still catching a real regression in either the single-format or the
    // retry-loop path.
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])),
    library_benchmark_groups = parse_exact_group
);
