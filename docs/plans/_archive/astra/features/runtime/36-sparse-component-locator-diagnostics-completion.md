---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/60/2026-08-28-sparse-component-locator-pages.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
related_code:
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse/locator.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
tests:
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse/tests.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
  - tools/tests/test_runtime_sparse_component_locator_pages_contract.py
---

# Runtime60 Sparse Locator Diagnostics Completion

## Plan Completion List

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime60 RECS-P1-11 | Aggregate sparse locator entry, page, and modeled-byte snapshots through `ComponentStorage` | implemented_pending_validation | Focused source contract 5/5, Runtime static performance contracts 1143/1143, Rustfmt, and scoped diff checks pass. The two-owner Rust regression requires a 32 KiB modeled bound for two high-index owners; managed Runtime caller tests and Release p50/p95/p99 evidence remain pending. |

## Implementation Record

`SparseRowLocator` now exposes a read-only structural snapshot. `SparseComponentStorage` forwards
that snapshot, and `ComponentStorage` aggregates every sparse owner with saturating arithmetic.
The owner-level regression covers two independent high-index locators and verifies that all locator
entries, pages, and modeled retained bytes return to zero when both owners are empty.

The snapshot is intentionally cold-path only: insert, remove, and lookup behavior do not read or
update diagnostic counters. The modeled byte value is a structural capacity estimate, not allocator,
RSS, or product-memory evidence.

Tooling remains out of scope. This record is not promoted until the managed Windows Runtime caller
tests and Release workload provide the required p50/p95/p99 evidence.
