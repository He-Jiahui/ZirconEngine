# Runtime157 Native Plugin Discovery Hint Single Buffer

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime157-editor103-performance-batch-20260826dn-v1`

## Problem

Native plugin hot reload failure handling collected discovered plugin ids into a temporary vector,
joined an intermediate string, and then formatted the final discovery hint. Failed reload attempts
therefore performed two avoidable intermediate allocations before rollback diagnostics.

## Optimization

- Accept discovered plugin ids through their borrowed iterator without a concrete candidate type.
- Compute exact final capacity from cloned iterator metadata and append ids into one result buffer.
- Preserve the empty-discovery message, report order, prefix, and comma separators.

## Regression Contract

The shared `optimization_batch_20260826dn_` filter owns three Runtime tests: output behavior,
exact-capacity source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME157_NATIVE_PLUGIN_DISCOVERY_HINT_SINGLE_BUFFER_BENCH_V1`, renders 16,384 hints with 32
plugin ids per sample, removes two intermediate allocations per hint, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
