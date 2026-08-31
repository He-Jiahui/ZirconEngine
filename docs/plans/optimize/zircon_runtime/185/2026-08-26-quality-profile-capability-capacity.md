# Runtime185 Quality Profile Capability Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime185-editor131-performance-batch-20260826ep-v1`

## Problem

Runtime quality-profile capability projection can emit one anti-alias requirement and six Solari
requirements. The full profile grew its vector from four to eight entries, while profiles with no
strict requirements still need to remain allocation-free.

## Optimization

- Compute the exact 0, 1, 6, or 7 entry capacity from the anti-alias and Solari feature flags.
- Allocate the requirement vector once before preserving the existing unique insertion logic.
- Preserve requirement ordering, capability mapping, duplicate suppression, and empty-profile
  zero allocation.

## Regression Contract

The shared `optimization_batch_20260826ep_` filter owns three Runtime tests: full-profile behavior,
exact-capacity source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME185_QUALITY_PROFILE_CAPABILITY_CAPACITY_BENCH_V1`, writes seven real
`RenderFeatureCapabilityRequirement` values 65,536 times per sample, reduces full-profile vector
allocations from two to one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
