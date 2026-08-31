# Runtime Text paragraph-analysis construction profile

Date: 2026-08-27

Status: `shape_request_analysis_profile_implemented / duplicate_construction_observable /
retained_paragraph_artifact_open / algorithm_unchanged / static_checks_complete /
managed_profile_pending`

## Scope

This record is the pre-optimization measurement slice for `RTS-P1-024`. It covers analysis objects
constructed inside one canonical shaping request:

- `BidiParagraph` for UAX #9 paragraph levels;
- `ParagraphTextAnalysis` for script segmentation and emoji-presentation ranges;
- `LineBreakOpportunityMap` for the direct or whole-alternate glyph projection.

It does not claim to measure every rich-layout, physical-line, word-boundary, retained-document, or UI
projection object. Those owners remain separate until a later operation/document-level profile can
attribute them without a process-global request history.

## Current call graph

`shape_text_with_diagnostics` canonicalizes the request, starts one profiling-only TLS aggregate, and
always detaches it after `cosmic::shape_text` returns Ready or an error. `BidiParagraph` and
`ParagraphTextAnalysis` are constructed once on the ordinary non-empty shaping path. Horizontal and
vertical direct paths each construct one `LineBreakOpportunityMap`. If horizontal direct does not
complete and whole-request Cosmic shaping is allowed, Cosmic currently constructs another line-break
map for the alternate glyph projection.

The second line-break construction is a real source-level duplication, but its cost and incidence are
not yet measured. Hoisting it into a shared request artifact now would change lifetimes and couple the
direct/alternate owners without proof that the work is material.

## Implemented measurement

The profiling leaf publishes eleven fixed counters:

- request count and request input bytes;
- Bidi build count, input bytes, and elapsed nanoseconds;
- script/emoji build count, input bytes, and elapsed nanoseconds;
- line-break build count, input bytes, and elapsed nanoseconds.

Constructor timing calls exist only in `test`, `profiling`, or `profiling-tracy` builds. Even in a
profiling build, `Instant::now` is skipped unless a managed capture has activated the request TLS.
Constructor leaves update integer TLS fields and the request owner publishes once, so scalar/grapheme
count does not become profiler lock count. No raw source, language, script, document ID, pointer, or
dynamic label is retained.

## Required dynamic profile

Use 1, 100, 1,000, and 10,000-grapheme inputs with 31 measured samples after explicit warm-up:

| Lane | Expected structural observation to verify |
|---|---|
| Horizontal direct success | one Bidi, one script/emoji, one line-break build |
| Horizontal whole-alternate | one Bidi, one script/emoji, two line-break builds |
| Horizontal hybrid candidate | same construction topology as whole-alternate; retain route receipt |
| Vertical direct success | one Bidi, one script/emoji, one line-break build |
| Stable-generation retry | all build counts scale with actual shaping-attempt count |
| Empty/missing-primary/error | record which constructors were reached; do not infer success work |

For each lane record wall p50/p95/p99, the eleven counters, direct/alternate route counters, request
resolution counters, cache-lock metrics, allocations, peak RSS, sampled CPU stacks, and available
Windows energy/power observation. Preserve glyph/layout hashes and typed outcome receipts so work
cannot be removed by changing behavior.

Only consider hoisting/reusing the line-break map if its measured time or allocation is material on the
alternate path. Only consider a retained paragraph artifact if combined duplicate construction across
shaping, rich layout, and physical-line owners is material under cold/warm and document-scale lanes.
The source lease, glyph storage, dirty-range dependency graph, and renderer artifact remain separate
decisions.

## Static evidence

- Eleven production names are fixed and unique under `text_analysis_*`.
- TLS aggregation has a focused regression for `1/1/2` Bidi/script/line-break construction and exact
  input-byte/nanosecond accumulation.
- Scoped Rust 2024 formatting and `git diff --check` pass.
- Touched production owners remain below 800 lines; the profiling leaf is folder-backed.

Managed Cargo, profiler capture, CPU sampling, timing/RSS/power, WGPU, and PNG were not run. No
performance improvement or accepted milestone is claimed.
