//! Gungraun (instruction-count) benchmark for the custom-format-string path of
//! `TimeSpan::to_string_format`/`TimeSpan::try_format`, both of which route through
//! `format_customized` (`src/time_span_format_custom.rs`) and its one intermediate
//! allocation — see issue #72 and `try_format`'s doc comment for the full rationale.
//!
//! The format string exercises every token kind `format_customized`'s tokenizer
//! dispatches on in one call: `'d'`, `'h'`/`'m'`/`'s'`, `'f'`, and the `'\\'` escape
//! branch.
use std::hint::black_box;

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError};
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};

/// Exercises 'd', 'h', 'm', 's', 'f', and '\\' (escaped literal separators) all in one
/// format string — see this file's module doc comment.
const CUSTOM_FORMAT: &str = "dd\\.hh\\:mm\\:ss\\.fffffff";

#[library_benchmark]
#[bench::custom_format(TimeSpan::from_hms(2, 3, 4).unwrap())]
fn bench_to_string_format(ts: TimeSpan) -> Result<String, TimeSpanError> {
    black_box(black_box(ts).to_string_format(black_box(CUSTOM_FORMAT)))
}

#[library_benchmark]
#[bench::custom_format(TimeSpan::from_hms(2, 3, 4).unwrap())]
fn bench_try_format(ts: TimeSpan) -> Result<usize, TimeSpanError> {
    // Buffer sized generously for "02.03:04.0000000" (well under 32 bytes) so the
    // benchmark measures `format_customized`'s allocation-then-copy cost, not
    // `InsufficientBuffer` bailout.
    let mut buf = [0u8; 32];
    black_box(black_box(ts).try_format(black_box(&mut buf), black_box(CUSTOM_FORMAT)))
}

library_benchmark_group!(
    name = format_customized_group,
    benchmarks = [bench_to_string_format, bench_try_format]
);

main!(
    // 5% is comfortably above callgrind's own binary-layout noise floor (a few tenths
    // of a percent) while still catching a real regression in the custom-format
    // tokenizer or the String-growth path it exercises.
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])),
    library_benchmark_groups = format_customized_group
);
