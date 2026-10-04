---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_parent_validation.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-detached-restore-union-cycle-rejection.md
tests:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_cycle_tests.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_cycle_profile.rs
---

# Runtime1014: detached restore union-cycle rejection

Status: `implemented_static_reviewed_managed_validation_pending`.

- [x] Bind the production successor to Runtime1012 SHA `add049519a0d2e14cbbc599cb34d39ae18412049b10d88a6f5822b365bcccc0c`; preserve its root normalization and all unrelated owner paths.
- [x] Add a local batch/live parent overlay check after all existing restore preflight errors. Distinguish prospective `HierarchyCycle` from reachable preexisting `HierarchyParentChainCycle` without scanning unrelated World entities.
- [x] Add real two-/three-edge, repeated-live-node, preexisting/unrelated cycle, missing-live-ancestor, pending query, error-priority, retry/ownership and valid multi-root regressions. Freeze the complete prior restore preflight as the Release baseline; record a real restore profile and reachable-parent read count.
- [x] Update the canonical detached-batch contract as an explicit successor of Runtime1013 SHA `687e3ccf8cb5e086338a7eaba3233cb10321b4157039b1095715e2c635192379`, preserving its prior contracts.
- [x] Complete independent source review, exact inverse, all path SHA and static receipts. No dynamic result is implied by formatting or source review.
- [ ] Run the managed full lib lane and grouped Release filters: `runtime62_detached_restore_union_cycle_release_profile`, `runtime62_detached_parent_cycle_release_profile`, `prepared_camera_subtree_managed_scale_fixture`, `detached_entity_batch_managed_scale_fixture`.

The lower deferred-flush walker and broader hierarchy authority remain independent Runtime62 acceptance work. Direct Cargo, validation submission, compile monitoring, commits and external notifications were not performed by this owner.
