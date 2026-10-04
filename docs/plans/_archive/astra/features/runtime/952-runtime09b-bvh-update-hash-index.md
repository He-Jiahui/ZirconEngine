---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-bvh-update-hash-index.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/planning/build_bvh_update_plan.rs
tests:
  - zircon_runtime/src/graphics/visibility/planning/build_bvh_update_plan.rs
---

# Runtime952 · BVH update hash indexes

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b BVH update planning | Sorted current/previous snapshots use a two-pointer delta merge for the normal stable-key path; unsorted inputs retain the two `HashMap<u64, &VisibilityHistoryEntry>` indexes as the fallback. Inserted and updated vectors still follow the current slice; removed vectors still follow the previous slice, and equality/full-rebuild behavior is unchanged. | The in-file order/source contracts cover all three delta classes and reject production ordered maps. Current-source Release owner evidence measured `legacy_p95_ns=9,995,900` versus `optimized_p95_ns=179,200` (`98.21%` reduction). Managed Cargo, allocator, and product gates remain pending. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/visibility/planning/build_bvh_update_plan.rs` | `CD5421AE0E389A0B5AD409AB8E7B841995B8317972E95858E331FB3EEFB63A1F` |

## Validation handoff

Exact-file formatting and scoped diff checks are part of the grouped Runtime
submission. Managed Cargo and the Release P95, allocator, and product gates
remain pending; no per-task managed run is started.
