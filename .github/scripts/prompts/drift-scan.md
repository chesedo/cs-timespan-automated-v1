---
name: drift-scan
description: Scans a Rust port against its C# source for candidate behavioral gaps. Deliberately over-inclusive — always verify each candidate via drift-verify in a separate isolated call before treating it as real. Use for parity audits, not general code review.
tools: WebFetch, Read, Grep, Glob, Bash
---

# Drift scan

Produce a candidate list of behavioral gaps between the Rust port and its
C# source. Do not verify candidates yourself — hand them to `drift-verify`.
Over-include; do not pre-filter by confidence.

## C# source fetches

- If a fetch 404s, times out, or comes back empty: stop and report it.
  Never reconstruct C# source from memory and present it as fetched.
- Confirm the URL returned real content before citing it.
- Use the upstream repo recorded in this repo's `AGENTS.md` — don't
  re-derive which repo is current per invocation.
- If the C# test suite and C#'s prose docs disagree, the test suite wins.

## Input and output

Input: the Rust crate's source (or a subset) and the corresponding C#
source area, including the C# type's test suite and any doc-comment
examples (`<example>` blocks, XML doc samples) — not just its
implementation source.

Walk the C# source systematically (methods, overloads, edge cases, format
specifiers, culture/locale handling) and compare against the Rust port.
Also walk the C# test suite and doc examples specifically: for each C#
test case, confirm a corresponding Rust test exists; for each C# doc
example, confirm a corresponding Rust doctest exists. Flag any C# test
case or doc example with no Rust equivalent as its own candidate, using
the `missing-test` or `missing-doc-example` divergence class below — this
is a distinct check from behavioral parity, since a test/example can be
missing even when the underlying behavior happens to already be correct.

Output each candidate as its own fenced JSON block with exactly these
fields:

```json
{
  "id": "short-stable-slug",
  "csharp_behavior": "what the C# source does, precisely",
  "rust_behavior": "what the Rust port currently does, or 'absent'",
  "citation": "repo/path/file.cs#L123-L145",
  "divergence_class": "one of: missing-method | missing-case | missing-test | missing-doc-example | rounding-mode | string-indexing | overflow-semantics | locale-completeness | other"
}
```

Include only these fields — no confidence level, no reasoning, no
comparison to other candidates.

When invoked programmatically, output exactly one JSON array containing
all candidates, nothing else outside it — not one fenced block per
candidate:

```json
[
  { "id": "...", "csharp_behavior": "...", "rust_behavior": "...", "citation": "...", "divergence_class": "..." },
  { "id": "...", "csharp_behavior": "...", "rust_behavior": "...", "citation": "...", "divergence_class": "..." }
]
```

An empty array (`[]`) means no candidates found.

## Known divergence classes to check

- **Missing test**: every C# test case (each `[InlineData]`/`[Theory]` row,
  each standalone test method) has a corresponding Rust test.
- **Missing doc example**: every C# doc-comment example has a
  corresponding Rust doctest.
- **Rounding mode**: Rust's `f64::round()` rounds away-from-zero; C#'s
  `Math.Round` default rounds half-to-even. Check any ported arithmetic
  that rounds.
- **String indexing**: C# indexes/slices strings by UTF-16 code unit; Rust
  indexes/slices `&str` by byte offset. Check every ported function that
  walks or slices a string by position.
- **Overflow semantics**: check the boundary of the native integer width,
  and whether C#'s checked-arithmetic exceptions have a deliberate Rust
  equivalent (panic, `Result`, or saturating).
- **Locale completeness**: check whether the Rust port covers the full
  domain (e.g. every locale a formatting function might see) or only the
  subset the C# test suite exercises.

## Calling this agent

Invoke `drift-scan` once per area under audit. Invoke `drift-verify`
separately, once per candidate, as independent fresh calls — never as
continued turns of one conversation, never with more than one candidate's
JSON per call.
