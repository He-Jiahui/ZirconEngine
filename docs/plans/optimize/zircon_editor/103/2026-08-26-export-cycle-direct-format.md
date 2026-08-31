# Editor103 Export Cycle Direct Format

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime157-editor103-performance-batch-20260826dn-v1`

## Problem

Export pipeline dependency-cycle errors collected stage CLI ids into a temporary vector, joined an
intermediate string, and then copied that string through the final `fmt::Formatter`. Invalid export
plans paid two avoidable intermediate allocations while producing diagnostics.

## Optimization

- Write the dependency-cycle prefix, separators, and stage CLI ids directly to the formatter.
- Keep all other plan-error formatting unchanged.
- Preserve empty cycles, stage order, and stable CLI identifiers.

## Regression Contract

The shared `optimization_batch_20260826dn_` filter owns three Editor tests: output behavior,
direct-formatter source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR103_EXPORT_CYCLE_DIRECT_FORMAT_BENCH_V1`, formats 16,384 cycles with 32 stages per sample,
removes two intermediate allocations per format, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
