---
title: Runtime93 Borrowed Mesh Locator Cache
category: zircon_runtime
report_id: Runtime93-borrowed-mesh-locator-cache-2026-08-26
date: 2026-08-26
session_id: root-runtime93-borrowed-mesh-locator-cache-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime93 Borrowed Mesh Locator Cache

## Scope

This slice optimizes external-mesh deduplication while resolving model geometry. It removes an
owned string-key allocation from every model primitive that references an external mesh. It does
not change primitive order, locator equality, dependency-state capture, mesh loading, deformation
aggregation, local bounds, fallback primitives, geometry revisions, or public resource contracts.

## Change

- The per-resolution mesh cache now uses `&AssetUri` keys borrowed from the input model instead of
  allocating `String` keys with `locator.to_string()` for every primitive.
- Equal locators held by distinct `AssetReference` values still share one cache entry because
  borrowed keys retain `ResourceLocator` value equality and hashing.
- Cache misses still record the dependency state before loading the referenced mesh, and cached
  `None` results continue to suppress repeated failed loads within the same model resolution.
- A Rust regression locks equal-locator deduplication, while a Python source contract prevents
  owned locator keys from returning to the hot path.

## Deterministic Performance Evidence

The independent release model uses 4,096 unique locator strings across 131,072 model primitives,
with 21 alternating owned-key/borrowed-key sample pairs per run. Both paths include the same
capacity-sized hash-table allocation; only the cache-key ownership differs.

| Evidence | Owned string keys | Borrowed locator keys | Result |
|---|---:|---:|---:|
| Measured allocations | 131,073 | 1 | 99.999237% fewer |
| Run 1 P50 | 37.525 ms | 10.233 ms | 72.73% faster |
| Run 1 P95 | 56.745 ms | 15.368 ms | 72.92% faster |
| Run 2 P50 | 40.462 ms | 10.216 ms | 74.75% faster |
| Run 2 P95 | 123.562 ms | 36.437 ms | 70.51% faster |
| Run 3 P50 | 24.767 ms | 8.168 ms | 67.02% faster |
| Run 3 P95 | 39.170 ms | 13.915 ms | 64.48% faster |

The managed gate requires the exact measured allocation counts above, at least 99.9% allocation
reduction, at least 50% P50 improvement, and at least 25% P95 improvement.

## Acceptance

- `tools.tests.test_runtime93_borrowed_mesh_locator_cache_performance_contract` passes 3/3
  locally.
- Exact-file `rustfmt --check` and scoped `git diff --check` pass locally.
- The `runtime93_borrowed_mesh_locator_cache_deduplicates_equal_locators` regression, source
  contracts, formatting, allocation/timing model, and the other two Runtime93 slices are submitted
  together in one coordinator-managed three-task validation batch.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Runtime93 still needs production LOD selection and transition evidence, large-scene instancing and
skinning budgets, morph deformation profiling, collision-product integration, mesh streaming
residency pressure measurements, and Editor/game authoring acceptance.
