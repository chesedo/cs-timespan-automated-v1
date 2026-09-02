//! Gungraun (instruction-count) benchmark pinning `TimeSpan::try_format`'s documented
//! non-allocating claim for the three standard formats "c", "g", "G". `try_format`
//! dispatches these to `try_format_standard` (`src/time_span.rs`), which computes the
//! required output length up front and writes directly into the caller's `&mut [u8]`
//! with no intermediate allocation. `to_string_format` (`src/time_span.rs`), by
//! contrast, always builds and returns an owned `String` — via `Display`/`to_string`
//! for "c" and via `format_general` for "g"/"G". See issue #71.
//!
//! Each pair below runs the same `TimeSpan` value and format specifier through both
//! APIs. CI discovers and runs `benches/*_gungraun.rs` as a same-job base-vs-head diff
//! (see `.github/workflows/bench.yml`), and gungraun's `soft_limits` gate a benchmark
//! against *its own* prior instruction count, not against a different benchmark's
//! count — there's no cross-benchmark "A must stay below B" primitive here. So the way
//! this file protects `try_format`'s allocation-free edge is indirect but effective:
//! `bench_try_format`'s cases get a soft limit much tighter than `bench_to_string_format`'s.
//! Writing a handful of ASCII digits into a fixed buffer is a small, allocation-free,
//! and highly stable operation, so even a cheap malloc/free pair introduced by a future
//! refactor (e.g. one that made `try_format_standard` share machinery with
//! `format_customized`, which *does* allocate — see `try_format`'s doc comment) would
//! show up as a large, easily-caught relative jump on `bench_try_format` specifically.
//! `bench_to_string_format` keeps the same 5% threshold used by `benches/parse_gungraun.rs`
//! for its own, unavoidably-allocating baseline.
//!
//! The `TimeSpan` value used (`from_dhms_milli(1, 2, 3, 4, 500)`) has a nonzero day
//! component and a nonzero fractional-second component, so all three formats produce
//! non-trivial output — including the day segment and, for "c"/"G", the
//! fractional-seconds segment — rather than a short/degenerate case that could mask
//! allocation overhead behind a tiny fixed cost.
use std::hint::black_box;

use cs_timespan_automated_v1::TimeSpan;
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind, LibraryBenchmarkConfig};

/// 1 day, 2 hours, 3 minutes, 4.5 seconds — exercises the day segment and, for "c"/"G",
/// the fractional-seconds segment, in every case below.
fn sample() -> TimeSpan {
    TimeSpan::from_dhms_milli(1, 2, 3, 4, 500).unwrap()
}

#[library_benchmark(config = LibraryBenchmarkConfig::default()
    // Tight: `try_format_standard` writes a fixed handful of ASCII digits directly into
    // `destination` with no allocation, so its instruction count should barely move run
    // to run — a threshold this close to callgrind's own noise floor (a few tenths of a
    // percent) still catches even a single introduced malloc/free pair.
    .tool(Callgrind::default().soft_limits([(EventKind::Ir, 2.0)])))]
#[bench::c("c")]
#[bench::g("g")]
#[bench::g_upper("G")]
fn bench_try_format(format: &str) -> usize {
    let ts = black_box(sample());
    let mut buf = [0u8; 32];
    black_box(
        ts.try_format(black_box(&mut buf), black_box(format))
            .unwrap(),
    )
}

#[library_benchmark(config = LibraryBenchmarkConfig::default()
    // Same 5% used by benches/parse_gungraun.rs: `to_string_format` always allocates a
    // `String`, so this threshold just guards against *this* path regressing further,
    // not against it losing an allocation-free property it never claimed to have.
    .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])))]
#[bench::c("c")]
#[bench::g("g")]
#[bench::g_upper("G")]
fn bench_to_string_format(format: &str) -> String {
    let ts = black_box(sample());
    black_box(ts.to_string_format(black_box(format)).unwrap())
}

library_benchmark_group!(name = try_format_group, benchmarks = bench_try_format);
library_benchmark_group!(
    name = to_string_format_group,
    benchmarks = bench_to_string_format
);

main!(
    library_benchmark_groups = try_format_group,
    to_string_format_group
);
