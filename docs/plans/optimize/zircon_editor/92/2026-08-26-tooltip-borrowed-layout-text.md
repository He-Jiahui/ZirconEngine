# Editor92 Tooltip Borrowed Layout Text

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime146-editor92-performance-batch-20260826dc-v1`

## Problem

Retained tooltip layout copied trimmed title and body text into temporary `String` values for
height and width measurement. The same helpers were called again during paint, so each tooltip
frame allocated before reaching the paint command's required owned-string boundary.

## Optimization

- Return trimmed title and body as borrowed slices, including the static title fallback.
- Measure tooltip width and height directly from borrowed node storage.
- Allocate only when constructing an owning `HostPaintCommand`, preserving command lifetime and
  rendering behavior.

## Regression Contract

The shared `optimization_batch_20260826dc_` filter owns three Editor tests: trim/fallback behavior,
node-storage borrowing plus ownership-boundary source shape, and an ignored paired release P50/P95
benchmark. The benchmark emits `EDITOR92_TOOLTIP_BORROWED_TEXT_BENCH_V1`, resolves 262,144 text
slices per sample, records layout/helper allocations from 262,144 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
