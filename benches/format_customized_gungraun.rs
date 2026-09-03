//! Gungraun (instruction-count) benchmark for the custom-format-string path of
//! `TimeSpan::to_string_format`/`TimeSpan::try_format`, both of which route through
//! `format_customized` (`src/time_span_format_custom.rs`) and its one intermediate
//! allocation — see issue #72 and `try_format`'s doc comment for the full rationale.
//!
//! Two axes are exercised:
//! - format-string variety (`_by_format` benchmarks) on one ordinary value, isolating
//!   each token kind `format_customized`'s tokenizer dispatches on (`'d'`, `'h'`/`'m'`/
//!   `'s'`, `'f'`, quoted literals, and the `'\\'` escape branch) so a regression
//!   isolated to one token kind doesn't hide inside one aggregate number.
//! - value variety (`_by_value` benchmarks) at `TimeSpan::MIN`/`MAX` under the same
//!   aggregate "everything" format used by the `_by_format` cases, since no other
//!   benchmark in this crate exercises those boundary values.
use std::hint::black_box;

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError};
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};

const SAMPLE: fn() -> TimeSpan = || {
    TimeSpan::builder()
        .days(1)
        .hours(2)
        .minutes(3)
        .seconds(4)
        .milliseconds(500)
        .build()
        .unwrap()
};

/// Exercises 'd', 'h', 'm', 's', 'f', and '\\' (escaped literal separators) all in one
/// format string — the same aggregate format the old single-case benchmarks used,
/// kept here as the `everything` case among several now.
const EVERYTHING_FORMAT: &str = r"dd\.hh\:mm\:ss\.fffffff";

#[library_benchmark]
#[bench::digits_only(r"dd\.hh\:mm\:ss")]
#[bench::fraction_only(r"s\.ffffff")]
#[bench::quoted_literal(r"mm\:ss' min'")]
#[bench::everything(EVERYTHING_FORMAT)]
fn bench_to_string_format_by_format(fmt: &str) -> Result<String, TimeSpanError> {
    black_box(black_box(SAMPLE()).to_string_format(black_box(fmt)))
}

#[library_benchmark]
#[bench::digits_only(r"dd\.hh\:mm\:ss")]
#[bench::fraction_only(r"s\.ffffff")]
#[bench::quoted_literal(r"mm\:ss' min'")]
#[bench::everything(EVERYTHING_FORMAT)]
fn bench_try_format_by_format(fmt: &str) -> Result<usize, TimeSpanError> {
    // Buffer sized generously for all four format strings' output on `SAMPLE()` (the
    // longest, "everything", produces "01.02:03:04.5000000" at 19 bytes) so the
    // benchmark measures `format_customized`'s allocation-then-copy cost, not an
    // `InsufficientBuffer` bailout.
    let mut buf = [0u8; 32];
    black_box(black_box(SAMPLE()).try_format(black_box(&mut buf), black_box(fmt)))
}

#[library_benchmark]
#[bench::min(TimeSpan::MIN)]
#[bench::max(TimeSpan::MAX)]
fn bench_to_string_format_by_value(ts: TimeSpan) -> Result<String, TimeSpanError> {
    black_box(black_box(ts).to_string_format(black_box(EVERYTHING_FORMAT)))
}

#[library_benchmark]
#[bench::min(TimeSpan::MIN)]
#[bench::max(TimeSpan::MAX)]
fn bench_try_format_by_value(ts: TimeSpan) -> Result<usize, TimeSpanError> {
    // Custom-format strings never emit a sign character (see the doc comment on
    // `format_customized` in `src/time_span_format_custom.rs`: day/time magnitudes are
    // computed *before* negation, unlike `Display`/"g"/"G"), so both MIN and MAX render
    // their magnitude only: "10675199.02:48:05.4775808" (MIN) / "...4775807" (MAX), each
    // 25 bytes — well under 32.
    let mut buf = [0u8; 32];
    black_box(black_box(ts).try_format(black_box(&mut buf), black_box(EVERYTHING_FORMAT)))
}

library_benchmark_group!(
    name = format_customized_group,
    benchmarks = [
        bench_to_string_format_by_format,
        bench_try_format_by_format,
        bench_to_string_format_by_value,
        bench_try_format_by_value,
    ]
);

main!(
    // 5% is comfortably above callgrind's own binary-layout noise floor (a few tenths
    // of a percent) while still catching a real regression in the custom-format
    // tokenizer or the String-growth path it exercises.
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])),
    library_benchmark_groups = format_customized_group
);
