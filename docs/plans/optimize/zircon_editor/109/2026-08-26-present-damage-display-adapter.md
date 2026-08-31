# Editor109 Present Damage Display Adapter

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime163-editor109-performance-batch-20260826dt-v1`

## Problem

Verbose softbuffer present logging formatted a damage rectangle into an intermediate `String` and
then copied it into the complete log line. Full repaint fallback also allocated a standalone
`full` string before the final log allocation.

## Optimization

- Represent optional damage as a private borrowed `Display` adapter.
- Write rectangle coordinates or the `full` fallback directly into the final log formatter.
- Preserve one-decimal frame formatting and existing log fields.

## Regression Contract

The shared `optimization_batch_20260826dt_` filter owns three Editor tests: summary behavior,
Display/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR109_PRESENT_DAMAGE_DISPLAY_ADAPTER_BENCH_V1`, formats 131,072 full-repaint damage lines per
sample, reduces allocations from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
