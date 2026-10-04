---
title: Runtime207 Empty Dependency Readiness Fast Path
category: zircon_runtime
report_id: Runtime207-empty-readiness-fast-path-2026-09-09
date: 2026-09-09
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime207 Empty Dependency Readiness Fast Path

## Scope

`ProjectAssetManager::readiness_report` commonly serves leaf assets whose resource record has no
dependencies. The collector previously constructed the result vector, discovered set, and queue
even for that empty case. This slice returns immediately for an empty direct-dependency slice;
non-empty breadth-first traversal and its duplicate, cycle, missing-row, depth, and directness
semantics are unchanged.

## Implementation

`collect_dependency_readiness` now checks `dependency_ids.is_empty()` before calculating capacities
or constructing traversal containers. The empty result is `Vec::new()` with zero capacity, so leaf
asset reports do not pay for three traversal allocations. The existing Runtime207 enqueue-time
deduplication remains the owner for non-empty graphs.

## Local evidence

- `optimization_batch_20260909_runtime207_empty_dependencies_skip_traversal_allocations` checks
  guard ordering and the zero-capacity empty result.
- The merged Runtime/Editor performance-contract batch passed `1723/1723` (`Runtime 1143/1143`,
  `Editor 580/580`) after the regression-test source correction.
- The existing Runtime207 cyclic/shared/missing graph oracle remains unchanged and continues to
  compare the optimized collector with the legacy implementation.
- Scoped Rust formatting and source diff checks are required before any managed run.

This is a source-level allocation-bound improvement. Managed Windows Cargo, release CPU/allocation
measurements, and product-scale p50/p95/p99 remain pending; no performance pass is inferred from
the structural guard alone.

## Source fingerprint

- `zircon_runtime/src/asset/facade/readiness.rs`:
  `EE6A43F467AE6C7B9C84FD75F8C8350EC366EF30E8110B8A4B4C3D51EEC7473A`
- `zircon_runtime/src/asset/facade/readiness/capacity_tests.rs`:
  `4BD4A1B680D492FE492AFEB6E0A1DA933FCB9D75319661A1A9435A56AC1F7F5E`
- `zircon_runtime/src/asset/facade/readiness/astra_traversal_tests.rs`:
  `13CDC52F6E6BF8024F009557C3CC85021DE87E5CDABAC1127074ACF554E91666`
