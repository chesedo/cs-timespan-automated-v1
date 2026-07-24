# Agent instructions

## Publishing status

Not published to crates.io, no released versions. Breaking API changes
are acceptable — no backwards-compatibility shims or deprecation layers;
just change the API and update all call sites. Priority is matching
System.TimeSpan's behavior exactly; make a breaking change if closing a
gap requires one. Version stays at `0.1.0` until actually published.

(Delete this section, or update it, once cs-timespan-automated-v1 has a released
version.)

## Never fabricate data

If a fetch, read, or lookup fails or comes back empty, say so and stop —
never invent content to fill the gap. Applies especially to C# source:
don't reconstruct a function from memory and present it as fetched.

If the source is reachable but something's off (missing code, C# behavior
contradicting its own docs), say so explicitly.

## C# source citations

Cite the upstream source file and line number(s) when mirroring specific
C# behavior (e.g. `// src/libraries/System.Private.CoreLib/src/System/TimeSpan.cs#L338`).

Prefer the citation directly above the test that exercises the behavior,
not inline in the implementation. Only cite in `src/*.rs` when no test
covers the behavior.

When a Rust test duplicates a specific C# test case, cite the file and
line(s) of that case directly above the test function.

### Verify citations before trusting them

Before relying on a citation — existing or new:
1. Confirm it's reachable: fetch the URL/file and line.
2. Confirm it supports the claim: the cited C# says what the comment/test
   claims, and the Rust code behaves that way. Prove it empirically where
   feasible.

Cite the current upstream repo (`dotnet/runtime`), not a stale
mirror or archived predecessor. When the C# test suite and C#'s prose
docs disagree, the test suite wins.

When auditing citations in bulk, have a subagent independently re-verify
each one against live upstream source rather than trusting an earlier
summary. See `drift-scan`/`drift-verify` for the same principle applied
to finding gaps.

## Test coverage parity

When porting a C# test suite's data-driven test
(`[Theory]`/`[MemberData]`/`[InlineData]`), match **all** upstream cases,
including combinatorially-generated sets — port every generated row, not
a hand-picked subset. If a case is genuinely infeasible to port, state
that explicitly in a comment rather than omitting it silently.

### Doctests are not where that parity coverage goes

Doctests are short, illustrative usage examples — one happy-path doctest
per public method at most. Exhaustive edge-case/overflow coverage belongs
in integration tests, not doctests.

Don't trim a value out of an existing citation-backed parity test array
because a new doctest also demonstrates it — the array mirrors specific
upstream `InlineData` rows; removing a value breaks that mirror silently.
Duplication between a doctest and an integration test case is fine.

Write a doctest for every public method as its own API-review pass: if
writing one feels awkward, that's a finding about the API's shape.

## Nix

`flake.nix` is the source of truth for checks — `.github/workflows/ci.yml`
runs it via `nix flake check`, covering `cargo fmt --check`, `cargo
clippy --all-features -- -D warnings`, and `cargo test --all-features`.

- Run `nix flake check` before opening a PR.
- `cargo fmt` / `cargo clippy --all-features` / `cargo test --all-features`
  directly are fine for a faster inner loop, but confirm with `nix flake
  check` before considering work done — it includes `--all-features`,
  which plain `cargo clippy`/`cargo test` skip by default.
- `nix develop` drops into a devShell with the pinned Rust toolchain.

## Panics in library code

Bare `.unwrap()` and direct slice/index panics in parsing or public-API
code paths become `Result`-returning equivalents unless the panic is
truly unreachable — verify what happens on the edge-case input before
calling it unreachable. See `idiomatic-rust-review` for the fuller
checklist.

## Corrections

When the user corrects an approach or decision, consider recording it
here. If the correction is about something a skill
(`.claude/skills/*/SKILL.md`) did or instructed, ask whether the skill
file itself should be updated.
