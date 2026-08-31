# Editor126 Host Window Diagnostic Drain Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime180-editor126-performance-batch-20260826ek-v1`

## Problem

When the retained-host diagnostic queue had evicted entries, draining first collected exactly the
retained queue into a vector and then appended a synthesized warning. The append could grow and copy
the entire 64-entry output even though the final element count was known before draining.

## Optimization

- Take and reset the dropped-entry count before moving queue contents.
- Reserve retained diagnostics plus the optional warning slot in one output allocation.
- Extend directly from the queue drain and preserve warning order, text, severity, and reset
  behavior.

## Regression Contract

The shared `optimization_batch_20260826ek_` filter owns three Editor tests: eviction-report
behavior, capacity-aware source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `EDITOR126_HOST_WINDOW_DIAGNOSTIC_DRAIN_CAPACITY_BENCH_V1`, drains 64 diagnostic-size
values 8,192 times per sample, reduces output allocations from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
