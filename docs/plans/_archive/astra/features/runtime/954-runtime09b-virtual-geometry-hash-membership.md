---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-virtual-geometry-hash-membership.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build/hash_membership_tests.rs
tests:
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build/hash_membership_tests.rs
---

# Runtime954 · virtual-geometry hash membership

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b virtual-geometry request and hot-resident classification | Priority-preserving requested-page dedup uses a capacity-reserved `HashSet` paired with the first-seen vector, and hot-resident classification builds one evictable-page hash index. Publication order and the frozen/normal planning branches remain unchanged. | The source/behavior contracts reject the two production linear membership scans. Current-source Release owner evidence measured prioritized `2,345,700ns→264,900ns` (`88.71%` reduction) and hot-resident `473,300ns→160,300ns` (`66.13%` reduction); managed Cargo, allocator, and product gates remain pending. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build.rs` | `7CE901DECBE0BB2B0426E606859611FDCDF3431C700EF485DF3D6D0B49388EF3` |
| `zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build/hash_membership_tests.rs` | `D7EBBDB6C6537DDEC485EB1B1F146D7D023D92C41371EFA5EC4237B43C72F3EC` |

## Validation handoff

The owner is included in the grouped Runtime Release selector. Managed Cargo,
Release P95, allocator, and product gates remain pending; tooling remains
deferred.
