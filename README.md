# cs-timespan-automated-v1

A Rust port of [`System.TimeSpan`](https://learn.microsoft.com/en-us/dotnet/api/system.timespan) from [dotnet/runtime](https://github.com/dotnet/runtime), matching its behavior exactly rather than reinventing a duration type from scratch.

`TimeSpan` represents a signed, tick-precision (100ns) span of time — the same value semantics as C#'s type, ported line-by-line against the upstream source, with every constructor, arithmetic operation, parser, and formatter cited against the exact `TimeSpan.cs`/`TimeSpanParse.cs`/`TimeSpanFormat.cs` code it mirrors.

## Status

Not published to crates.io; no released versions yet. The public API can still change — see [`AGENTS.md`](AGENTS.md) for the exact policy. Priority is matching `System.TimeSpan`'s behavior exactly, including its documented edge cases; a breaking change is acceptable if closing a behavioral gap requires one.

**Deliberately out of scope, everywhere in this crate:** `IFormatProvider`/culture-aware parsing and formatting. Every parser and formatter here is invariant-culture-only — no method takes a culture parameter, and none will vary output by locale. This is a permanent design boundary, not a gap waiting to be filled.

## Usage

```rust
use cs_timespan_automated_v1::{TimeSpan, TimeSpanStyles};

// Construction
let ts = TimeSpan::from_hms(1, 2, 3).unwrap(); // 1h 2m 3s
let day_and_a_half = TimeSpan::from_days(1.5).unwrap();
assert_eq!(TimeSpan::from_ticks(36_000_000_000), TimeSpan::from_hms(1, 0, 0).unwrap());

// Component and total accessors
assert_eq!(ts.hours(), 1);
assert_eq!(ts.minutes(), 2);
assert_eq!(day_and_a_half.total_hours(), 36.0);

// Arithmetic (checked — returns Result rather than panicking on overflow)
let sum = ts.checked_add(TimeSpan::from_hms(0, 30, 0).unwrap()).unwrap();
let doubled = ts.checked_mul(2.0).unwrap();

// Parsing (invariant culture only)
let parsed: TimeSpan = "1.02:03:04.5000000".parse().unwrap();
let exact = TimeSpan::parse_exact("02:03:04", r"hh\:mm\:ss", TimeSpanStyles::None).unwrap();

// Formatting
assert_eq!(ts.to_string(), "01:02:03"); // "c" format (Display)
let general = ts.to_string_format("g").unwrap();

// Non-allocating formatting into a caller-supplied buffer
let mut buf = [0u8; 32];
let written = ts.try_format(&mut buf, "c").unwrap();
assert_eq!(&buf[..written], b"01:02:03");
```

Every public method has its own doctest demonstrating real, verified usage — `cargo doc --open` or browse `src/time_span.rs` directly for the full API surface.

## Development

This project uses [Nix](https://nixos.org/) as the source of truth for its check gate — `.github/workflows/ci.yml` runs it in CI.

```sh
nix flake check     # fmt --check, clippy --all-features -D warnings, test --all-features
nix develop         # drop into a devShell with the pinned Rust toolchain
```

Without Nix, the equivalent commands are `cargo fmt --check`, `cargo clippy --all-features -- -D warnings`, and `cargo test --all-features` — fine for a faster inner loop, but confirm with `nix flake check` before considering work done, since it's the one that runs with `--all-features` by default.

### Benchmarks

Performance-sensitive paths (parsing, formatting) have [`gungraun`](https://crates.io/crates/gungraun) benchmarks under `benches/`, measuring deterministic instruction counts (via valgrind/callgrind) rather than noisy wall-clock time. `.github/workflows/bench.yml` runs them on every PR touching `src/`, comparing the PR head against its base branch and failing on a real regression.

```sh
nix develop --command cargo bench   # run all gungraun benchmarks locally
```

## How this port is maintained

- [`AGENTS.md`](AGENTS.md) — conventions for citing C# source, verifying citations, and test-coverage parity.
- `.github/workflows/drift-check.yml` — a scheduled scan comparing this crate's behavior against the current upstream `dotnet/runtime` source, filing an issue for any real behavioral gap it finds (labeled `timespan-drift`).
- Issues filed this way (or manually) get worked end-to-end — verified, fixed or explicitly marked out of scope, and shipped as a narrowly-scoped PR — following the same conventions a human contributor would.

## License

Not yet licensed for reuse — no `LICENSE` file exists in this repository yet.
