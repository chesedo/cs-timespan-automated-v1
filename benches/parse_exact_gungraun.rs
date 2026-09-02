//! Gungraun (instruction-count) benchmark for `TimeSpan::parse_exact`/`parse_exact_multiple`,
//! covering each of the three standard formats (`"c"`/`"g"`/`"G"`, `src/time_span_parse_constant.rs`
//! and `src/time_span_parse.rs`) and the custom-format tokenizer (`src/time_span_parse_exact.rs`)
//! separately, each with several realistic input cases rather than one synthetic aggregate -- so a
//! regression isolated to one format or one token kind is visible instead of averaged away. See
//! issue #73 for the original file, #94 for the sibling lenient-`Parse` benchmark, and #95 for
//! this split.
use std::hint::black_box;

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError, TimeSpanStyles};
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};

#[library_benchmark]
#[bench::hms("12:24:02")]
#[bench::d_dot_hms("1.12:24:02")]
#[bench::with_fraction("1.12:24:02.9990000")]
#[bench::negative("-01.07:45:16.9990000")]
#[bench::max("10675199.02:48:05.4775807")]
#[bench::min("-10675199.02:48:05.4775808")]
fn bench_parse_exact_c(s: &str) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact(
        black_box(s),
        black_box("c"),
        TimeSpanStyles::None,
    ))
}

#[library_benchmark]
#[bench::days_only("42")]
#[bench::hm("12:34")]
#[bench::hms_frac("12:24:02.999")]
#[bench::d_hms_frac("1:12:24:02.999")]
#[bench::negative("-01:07:45:16.999")]
#[bench::min("-10675199:2:48:05.4775808")]
fn bench_parse_exact_g(s: &str) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact(
        black_box(s),
        black_box("g"),
        TimeSpanStyles::None,
    ))
}

#[library_benchmark]
#[bench::zero("0:00:00:00.0000000")]
#[bench::common("1:12:24:02.9990000")]
#[bench::max("10675199:02:48:05.4775807")]
#[bench::negative("-1:07:45:16.9990000")]
#[bench::min("-10675199:02:48:05.4775808")]
fn bench_parse_exact_g_upper(s: &str) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact(
        black_box(s),
        black_box("G"),
        TimeSpanStyles::None,
    ))
}

#[library_benchmark]
#[bench::dd_h_m_s("12.23:32:43", r"dd\.h\:m\:s")]
#[bench::ddd_h_m_s_fff("012.23:32:43.893", r"ddd\.h\:m\:s\.fff")]
#[bench::d_hh_mm_ss("12.05:02:03", r"d\.hh\:mm\:ss")]
#[bench::quoted_literal("12d23h32m43s", "d'd'h'h'm'm's's'")]
fn bench_parse_exact_custom(s: &str, fmt: &str) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact(
        black_box(s),
        black_box(fmt),
        TimeSpanStyles::None,
    ))
}

const MULTI_FORMATS: [&str; 3] = ["c", "g", "G"];

// `first_match` ("12:24:02") matches "c" on the first attempt. `last_match`
// ("1:12:24:02.9990000") fails "c" (which requires a '.' days/hours separator, not ':') but
// already matches "g" on the second attempt -- it never reaches "G". That's not a shortcut
// taken here: "g" and "G" (src/time_span_parse.rs::parse_with_style) share the exact same
// grammar for a full 5-number d:h:m:s.f shape, and "G"'s only extra constraint (require_full)
// *rejects* inputs with fewer than 5 numbers -- it adds no constraint "g" doesn't already
// enforce once 5 numbers are present. So any input "G" accepts, "g" accepts too, making a
// "fails c and g, matches only G" case structurally impossible for this parser; two candidates
// tried (rather than three) is the most this array can exercise before a match.
#[library_benchmark]
#[bench::first_match("12:24:02")]
#[bench::last_match("1:12:24:02.9990000")]
fn bench_parse_exact_multiple(s: &str) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact_multiple(
        black_box(s),
        black_box(&MULTI_FORMATS),
        TimeSpanStyles::None,
    ))
}

library_benchmark_group!(
    name = parse_exact_group,
    benchmarks = [
        bench_parse_exact_c,
        bench_parse_exact_g,
        bench_parse_exact_g_upper,
        bench_parse_exact_custom,
        bench_parse_exact_multiple,
    ]
);

main!(
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])),
    library_benchmark_groups = parse_exact_group
);
