# Runtime142 HTML Keyword Zero-Allocation Parse

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime142-editor88-performance-batch-20260826cy-v1`

## Problem

Bounded HTML rich-text parsing allocated lowercase strings for inline baseline and font-weight
attributes, then allocated another lowercase string for every text-decoration token. These values
come from document content and are parsed repeatedly across rich-text spans.

## Optimization

- Trim once and compare baseline/font-weight keywords with borrowed `eq_ignore_ascii_case` calls.
- Preserve numeric font-weight parsing and its inclusive `1..=1000` bound.
- Match each whitespace-delimited decoration token without allocating a lowercase copy.
- Preserve mixed-case `baseline/center/top/bottom`, `normal/bold`, `underline/line-through/none`,
  unknown-token handling, and decoration state ordering.

## Regression Contract

The shared `optimization_batch_20260826cy_` filter owns three Runtime tests: baseline/font-weight
behavior, decoration state behavior, and an ignored paired release P50/P95 benchmark. The benchmark
emits `RUNTIME142_HTML_KEYWORD_ZERO_ALLOCATION_PARSE_BENCH_V1`, parses 16,384 attribute groups,
records keyword allocations from 65,536 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
