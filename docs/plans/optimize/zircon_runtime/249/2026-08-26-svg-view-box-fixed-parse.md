# Runtime249 SVG View Box Fixed Parse

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime249-editor195-performance-batch-20260826hc-v1`

## Problem

SVG icon `viewBox` parsing collected every successfully parsed number into a heap-backed vector and
then accepted the vector only when it held exactly four values. Every icon parse therefore allocated
a dynamic collection for a format whose arity is fixed by definition.

## Optimization

- Parse successful numeric tokens directly into a four-slot stack array.
- Reject a fifth valid number immediately without growing a collection.
- Preserve the existing invalid-token filtering and exactly-four-valid-values acceptance contract.

## Regression Contract

The `optimization_batch_20260826hc_` Runtime tests preserve whitespace/comma parsing, missing and
extra values, and invalid-token behavior; enforce fixed-slot parsing; and provide an ignored paired
release benchmark emitting `RUNTIME249_SVG_VIEW_BOX_FIXED_PARSE_BENCH_V1`. It repeatedly parses a
four-value `viewBox` and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
