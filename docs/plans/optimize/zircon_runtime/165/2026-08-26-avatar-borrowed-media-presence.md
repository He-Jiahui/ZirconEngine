# Runtime165 Avatar Borrowed Media Presence

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime165-editor111-performance-batch-20260826dv-v1`

## Problem

MUI Avatar class generation cloned the first non-empty media attribute only to decide whether the
default-color class was required. The temporary string did not escape the presence check.

## Optimization

- Borrow and trim the first non-empty media string directly from the node attribute map.
- Preserve alias order, whitespace-only handling, and default-color behavior.
- Keep the helper local to the display-surface class owner.

## Regression Contract

The shared `optimization_batch_20260826dv_` filter owns three Runtime tests: alias/empty behavior,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME165_AVATAR_BORROWED_MEDIA_PRESENCE_BENCH_V1`, performs 524,288 lookups per sample, reduces
lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
