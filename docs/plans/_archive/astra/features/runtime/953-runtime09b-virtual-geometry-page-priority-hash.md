---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-virtual-geometry-page-priority-hash-aggregation.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/frontier/unique_pages.rs
tests:
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/frontier/unique_pages.rs
---

# Runtime953 · virtual-geometry page-priority hash aggregation

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b virtual-geometry streaming frontier | Page-priority aggregation uses a dense page-ID vector when IDs fit a bounded range and a capacity-reserved `HashMap<u32, PagePriority>` for sparse IDs. Resident filtering and the explicit cluster-count, error, LOD, cluster-ID, and page-ID ranking comparator remain unchanged. | Ranking, resident-page, zero-budget, and source contracts are in-file. Current-source Release owner evidence measured `legacy_p95_ns=13,133,000` versus `hash_p95_ns=2,134,500` (`83.75%` reduction). Managed Cargo, allocator, and product gates remain pending. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/frontier/unique_pages.rs` | `B139BF252FF9577DC03DE80365C95FF41C39232291F0145D03DF85603B8AD7FB` |

## Validation handoff

The owner is included in the grouped Runtime Release selector. Managed Cargo,
Release P95, allocator, and product gates remain pending; tooling remains
deferred.
