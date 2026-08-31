# Editor167 Activity Rail Profile Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime221-editor167-performance-batch-20260826fz-v1`

## Problem

Editor profiling rebuilt the combined left/right activity-rail button frame list from an empty Vec,
despite both retained models exposing their row counts before collection starts.

## Optimization

- Sum the left and right rail row counts with saturation and preallocate the combined output Vec.
- Preserve left-before-right ordering, visibility filtering, translated frames, identities, labels,
  and empty-rail behavior; filtered rows may leave spare capacity without extra allocations.

## Regression Contract

The `optimization_batch_20260826fz_` Editor tests cover combined capacity, values, saturating count
composition, and the production reserve contract, and provide an ignored paired release benchmark
emitting `EDITOR167_ACTIVITY_RAIL_PROFILE_CAPACITY_BENCH_V1`. It builds 128 frames containing 4,096
profile-sized button payloads per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
