# Editor105 Reference Palette Label Borrowed Format

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime159-editor105-performance-batch-20260826dp-v1`

## Problem

UI asset palette rebuild copied either a reference component suffix or the full reference into an
intermediate string, then copied that value again into the final `Reference / ...` label. Imported
component catalogs paid two allocations for each reference row.

## Optimization

- Select the component suffix or full reference as a borrowed `&str`.
- Format the final palette label directly from that borrowed slice.
- Preserve fragment, fragment-free, and empty-fragment labels without changing reference keys.

## Regression Contract

The shared `optimization_batch_20260826dp_` filter owns three Editor tests: label behavior,
borrowed source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR105_REFERENCE_PALETTE_LABEL_BORROWED_FORMAT_BENCH_V1`, renders 131,072 labels per sample,
records allocations per label from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
