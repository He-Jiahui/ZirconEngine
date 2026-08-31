# Runtime158 Rigid Body Enum Borrowed Match

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime158-editor104-performance-batch-20260826do-v1`

## Problem

Rigid-body reflection writes normalized every incoming enum name into a newly allocated lowercase
alphanumeric `String` before matching mass mode, CCD mode, or sleep policy. Scene/property updates
paid that allocation even though all expected values are fixed ASCII tokens.

## Optimization

- Compare a filtered lowercase byte iterator directly with borrowed expected-token bytes.
- Share the zero-allocation matcher across all three reflected enum fields.
- Preserve punctuation/whitespace filtering, ASCII case folding, and invalid-value diagnostics.

## Regression Contract

The shared `optimization_batch_20260826do_` filter owns three Runtime tests: normalization behavior,
borrowed source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME158_RIGID_BODY_ENUM_BORROWED_MATCH_BENCH_V1`, performs 262,144 matches per sample, records
allocations per match from one to zero, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
