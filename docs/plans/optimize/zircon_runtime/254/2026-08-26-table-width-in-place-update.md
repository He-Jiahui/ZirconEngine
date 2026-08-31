# Runtime254 Table Width In-Place Update

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime254-editor200-performance-batch-20260826hh-v1`

## Problem

Every retained table-column resize converted borrowed field names into owned strings before
updating the column-width map. The matching column entry also allocated the stable `width` key,
so repeated drag updates discarded two unnecessary key allocations per event.

## Optimization

- Replace existing column-width values through a borrowed `get_mut` lookup.
- Reuse the same helper for the matched column's stable `width` property.
- Preserve owned-key insertion when a field or width property is introduced for the first time.

## Regression Contract

The `optimization_batch_20260826hh_` Runtime tests preserve existing and new field updates plus
matched-column behavior; enforce the borrowed lookup and new-key fallback; and provide an ignored
paired release benchmark emitting `RUNTIME254_TABLE_WIDTH_IN_PLACE_BENCH_V1`. It repeatedly updates
an existing width under a 32 KiB field name and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
