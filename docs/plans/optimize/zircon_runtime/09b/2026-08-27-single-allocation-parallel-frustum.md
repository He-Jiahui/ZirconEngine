---
title: Runtime09B Direct-Output Parallel Frustum
category: zircon_runtime
report_id: Runtime09B-direct-output-parallel-frustum-2026-08-27
date: 2026-08-27
session_id: root-runtime09b-dirty-proportional-static-index-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Direct-Output Parallel Frustum

## Scope

The parallel frustum path previously copied every candidate into a larger
`MeshFrustumWorkItem`, evaluated visibility in that array, and then projected the work array into
the final `MeshFrustumVisibility` vector. The path now uses the existing ordered
`parallel_map_indices` task primitive to evaluate a source candidate and construct its final result
at the same index.

The no-pool and below-64-candidate serial fast paths are unchanged. Parallel output order, stable
instance keys, camera-test construction count, and visibility results remain covered by the existing
parallel-versus-serial behavior test.

## Allocation Evidence

Rust's release `Vec::IntoIter::map().collect()` can reuse the old work-vector allocation for the
smaller result type. Therefore the prior path already performs one physical allocation, despite two
logical collections. This optimization does not claim an allocation-count reduction.

The representative model uses the production-equivalent layouts and 131,072 candidates:

| Metric | Work-item projection | Direct final output | Reduction |
|---|---:|---:|---:|
| physical allocations | 1 | 1 | 0% |
| allocated bytes | 4,194,304 | 2,097,152 | 50.000% |
| intermediate work-item projection pass | 1 | 0 | 100% |

## Timing Evidence

Each run uses 21 alternating sample pairs, identical visibility arithmetic, exact output equality,
and result checksum `5751860017608785243`.

| Run | Work-item P50 ns | Direct P50 ns | Reduction | Work-item P95 ns | Direct P95 ns | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 3,148,900 | 1,174,000 | 62.717% | 12,195,600 | 1,501,800 | 87.686% |
| 2 | 2,708,300 | 1,145,900 | 57.689% | 3,907,200 | 1,399,000 | 64.194% |
| 3 | 3,089,200 | 1,126,100 | 63.547% | 6,014,800 | 2,795,200 | 53.528% |
| 4 | 2,626,500 | 1,041,600 | 60.343% | 2,998,000 | 1,310,400 | 56.291% |

The four-run worst case reduces P50 by 57.689% and P95 by 53.528%. The model gate requires the
physical allocation count to remain one, allocated bytes to fall by at least 45%, P50 to fall by at
least 15%, P95 to fall by at least 5%, and both result and timing checksums to remain exact.

## Validation

Passed locally without Cargo:

- 3/3 Python source contracts for direct indexed output and removal of the intermediate work type;
- Rust 1.94.1 formatting check for the production source;
- scoped `git diff --check`;
- four independent optimized model runs with identical result/timing checksums and all gates met.

Managed validation must run the existing parallel/serial order-and-result Rust test, the camera-test
construction contract, the three Python contracts, formatting, scoped diff, and the release model in
one coordinator ticket. Cargo validation is not claimed before that asynchronous ticket passes.

## Remaining Parent-Plan Work

This slice only removes the parallel frustum work-item projection. BVH update policy, static-index
maintenance, per-view candidate construction, HZB occlusion, draw batching, and GPU-scene upload
remain owned by the Runtime09B parent plan.
