# Runtime Text missing-primary and generation-deferred capability receipts

Date: 2026-08-27

Scope: `RTS-P1-009`, `RTS-P1-013`, `RTS-P1-014`; non-validation production slice.

## Current-source finding

The fallback resolver already retained primary/fallback/partial/last-resort/depth-limit selection on
`FallbackTextSpan`, but failure to select any primary face was converted to a neutral
`TextLayoutError::FontUnavailable` before a request-owned receipt existed. Font-generation retry
exhaustion, stale retained-session cache/result, and stale parallel worker completion similarly
produced `FontGenerationChanged` without a stable shaping cause. The diagnostics consumer then used
the terminal-recording path for explicit deferred outcomes.

This lost two facts needed for a capability-oriented pipeline:

1. font selection never produced a primary face;
2. otherwise valid work must be retried against a newer font generation rather than repaired as a
   terminal shaping failure.

## Reference check

Local Unreal `FCompositeFontCache::GetFontDataForCodepoint` preserves the sub/default/fallback
selection until a concrete font-data result is chosen. `FSlateTextShaper` then retains face validity
and loading state, exposes loading faces to its caller, and uses an explicit substitute path when the
selected face is invalid or loading. Zircon does not copy the per-codepoint algorithm: it keeps its
grapheme coverage and cluster-single-face policy. The adopted invariant is narrower and structural:
font resolution/load state remains owned by the shaping request and is not reconstructed from empty
glyph output or a public error string.

## Implementation

- Stable failure codes append `FontPrimaryUnavailable` and `FontGenerationChanged`; the catalog and
  `ALL` table both contain 14 entries.
- Missing primary maps to phase `FontResolution`, dependency `FontDatabase`, terminal disposition,
  and neutral public error `FontUnavailable`.
- Generation instability maps to the same phase/dependency with deferred disposition and neutral
  public error `FontGenerationChanged`.
- Stable-generation retry exhaustion, stale retained-session cache/result, explicit worker defer,
  and stale parallel worker completion use the same generation receipt constructor.
- Session and parallel reports separate deferred failure/run counts from terminal counts. UI profile
  projection first added two fixed names. The subsequent request-resolution receipt expands the
  current projection to 35 session dimensions and 66 total layout-resolve emissions under capacity
  128; the broader integration capture is 160 after fixed cache-lock/analysis streams were added.
- Generation-focused tests live in the diagnostics leaf; the current `layout_session.rs` is 762 lines.

The subsequent request-resolution slice observes the existing fallback loops without changing their
candidate order, coverage result, span merge, backend call, cache key/admission, retry budget, or
shaping/layout algorithm. Runtime complexity and allocation scale are unchanged.

## Static evidence

- Rust 2024 rustfmt passes for the scoped affected Rust files.
- trailing whitespace: 0.
- explicit failure-code variants: 14.
- `TextShapingFailureCode::ALL` entries: 14.
- neutral `FontGenerationChanged.into()` shaping conversions: 0.
- current key owner sizes: layout session 762, failure receipt 541, model receipt 263, outcome 282,
  parallel pool 499, UI profile root/session leaf 694/193 lines.

Cargo, fault injection, concurrent generation stress, 31-sample timing/RSS/power, WGPU rendering,
and PNG evidence were not run. No performance, power, or product-acceptance claim is made.

## Remaining work

The full capability trace remains open. The current synchronous subset now preserves fixed aggregate
candidate cache/probe/rejection/selection counts in a request-owned transient envelope. A later
font-runtime-owned design must preserve bounded exact candidate identity/ordinal, pending dependency,
policy rejection, backend unsupported,
and generation identity without putting raw text, family names, or per-candidate dynamic labels into
the profiler. That work must be coordinated with Runtime Font plan 80 and measured before changing
the resolver algorithm.

Status: `primary_and_generation_capability_causes_implemented /
deferred_terminal_split_implemented / fixed_profile_projection_implemented /
algorithm_unchanged / static_checks_complete / managed_validation_pending /
full_capability_trace_open`.
