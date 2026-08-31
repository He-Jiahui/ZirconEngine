# Runtime181 Map Rename Single Resolution

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime181-editor127-performance-batch-20260826el-v1`

## Problem

Renaming a component-state map key resolved the same property map once for target-key validation,
again for source-key validation, and a third time for removal and insertion. Each resolution
repeated the outer state-value lookup and value-kind branch on the success path.

## Optimization

- Resolve the mutable property map once after the identity no-op check.
- Reuse that map for target conflict and source presence validation.
- Move the value and clear reference provenance without reacquiring the property.

## Regression Contract

The shared `optimization_batch_20260826el_` filter owns three Runtime tests: rename/error behavior,
single-resolution source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME181_MAP_RENAME_SINGLE_RESOLUTION_BENCH_V1`, performs 8,192 alternating renames against a
256-property state per sample, reduces property-map resolutions from three to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
