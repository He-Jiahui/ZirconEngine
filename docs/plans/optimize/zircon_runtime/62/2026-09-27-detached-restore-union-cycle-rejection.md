---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_parent_validation.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-detached-subtree-parent-cycle-rejection.md
tests:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_cycle_tests.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_cycle_profile.rs
---

# Runtime62 detached restore union-cycle rejection

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| Runtime1014 | Validate the combined batch and live parent graph before moving detached rows back into World | `implemented_static_reviewed_managed_validation_pending` | 2026-09-27 | [Astra checklist](../../../astra/features/runtime/1014-runtime62-detached-restore-union-cycle-rejection-completion-list.md); `2026-09-27-astra-optimize-batch-r-runtime62-restore-union-cycle-*` receipts |

## Failure and contract

A valid `R.parent=P` can be detached, after which the supported raw `get_mut::<Hierarchy>(P)` route can set `P.parent=R` while R is absent. The old restore preflight checks that the batch parent P exists, then restores R; the candidate union R→P→R is never checked. `HierarchyTopology::update_parent` does not validate this graph, and the scene-binding ancestor walk merely bounds traversal. A reachable live-only cycle is also accepted by the old preflight. An unrelated live cycle must not cause a whole-World scan or block an unrelated restore.

The restore preflight still checks empty/permutation/identity/capacity, duplicate or occupied live identity, archetype/table/sparse compatibility, each batch parent's existence, and active-camera validity in its original order. Only after those checks does it validate parent chains reachable from batch entries. Batch Hierarchy rows override live parent lookup for their IDs; other IDs read authoritative live Hierarchy rows. A visiting/completed memo shares ancestors across entries and traverses each reachable parent once. Raw missing ancestors outside the batch retain their previous terminal-chain semantics.

A cycle containing a batch row returns `HierarchyCycle { child, parent }` for a batch edge on that cycle. A reached cycle containing only preexisting live rows returns `HierarchyParentChainCycle { start, repeated }`. No new SceneError variant, full World clone, index rebuild, deferred notification flush, or global validity snapshot is introduced. A rejected restore changes only the existing rejected-preflight diagnostic, returns the original move-only batch, and allows repair followed by retry. Compare world facts, effective generation including pending query mutations, ticks, lifecycle, observers, stable rows, and topology against a snapshot taken after the raw edit.

This is an explicit successor of Runtime1012 `detached_entity_batch.rs` SHA `add049519a0d2e14cbbc599cb34d39ae18412049b10d88a6f5822b365bcccc0c`. Its prepare root-normalization and Q parent-chain implementation remain read-only. The canonical detached-batch document is a separate successor of Runtime1013 SHA `687e3ccf8cb5e086338a7eaba3233cb10321b4157039b1095715e2c635192379`; its prior Runtime1012/1013 contracts remain intact.

## Verification and remaining acceptance

- Real World regressions cover prospective two- and three-edge cycles, a cycle whose repeated node is live but another cycle edge belongs to the batch, reachable preexisting live cycles, unrelated live cycles, raw missing-live-ancestor compatibility, unflushed query mutation, missing-parent and duplicate-entity precedence, rejected-batch ownership and repair/retry, and valid multi-root restore preserving stable order and owned table/sparse/dynamic/observer payloads and change ticks.
- An ordinary scale regression counts one lookup per unique reachable live ancestor for 2 and 128 detached roots with a 2048-row shared chain and unrelated rows. It calls the same local validator on a real detached World and batch.
- The ignored Release fixture `runtime62_detached_restore_union_cycle_release_profile` freezes the complete previous preflight body. On valid forests with 2/128 roots, 4096 shared ancestors and 100000 unrelated rows, it measures five warmup and 31 alternating old/current preflight pairs plus actual restore calls. Setup, result checks and re-detach remain outside timing. Record raw nanoseconds and p50/p95/p99; the plan does not invent an absolute or percentage threshold.
- The grouped Release lane must include `runtime62_detached_restore_union_cycle_release_profile`, `runtime62_detached_parent_cycle_release_profile`, `prepared_camera_subtree_managed_scale_fixture`, and `detached_entity_batch_managed_scale_fixture`. Nonignored behavior tests belong to the full managed lib lane.

Static review, managed compilation, behavior tests and Release measurements are separate evidence. No Cargo or compiler was run while authoring this candidate. The lower deferred-flush subtree walker, protected Hierarchy mutation authority, and wider Runtime62 graph validity gates remain separate work.
