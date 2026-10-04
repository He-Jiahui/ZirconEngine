---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-particle-history-hash-dedup.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/build_history_snapshot.rs
tests:
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/build_history_snapshot.rs
---

# Runtime956 · particle-history hash deduplication

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b particle history snapshot | Large emitter batches take a contiguous `Vec` sort/dedup fast path; small batches retain `HashSet<EntityId>` admission before the final sort. The public history remains ascending and unique while ordered-tree admission is removed. | Unordered duplicate behavior and the production hash/sort contract pass structurally. Current-source Release owner evidence measured `legacy_p95_ns=2,728,200` versus `optimized_p95_ns=135,990` (`95.01%` reduction); managed Cargo, allocator, and product gates remain pending. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/build_history_snapshot.rs` | `862E3E82660D57507A727584134FBBA40385583148AC93CF2C145CA32748EEDC` |

## Validation handoff

The owner is included in the grouped Runtime Release selector. Managed Cargo,
Release P95, allocator, and product gates remain pending; tooling remains
deferred.
