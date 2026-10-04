---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/ecs/commands/command.rs
  - zircon_runtime/src/scene/ecs/commands/commands/checked_reparent.rs
  - zircon_runtime/src/scene/ecs/commands/commands/facade.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-protected-scene-authored-component-authority.md
tests:
  - zircon_runtime/src/scene/tests/ecs_commands/checked_reparent.rs
---

# Runtime62 deferred checked reparent command

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| Runtime1020 | Queue checked Scene hierarchy edits without generic `Hierarchy` mutation | `candidate_static_review_complete_managed_validation_pending` | 2026-09-27 | [Astra completion list](../../../astra/features/runtime/1020-runtime62-deferred-checked-reparent-command-completion-list.md); exact source and predecessor hash handoff |

## Observed gap and contract

The Runtime1017 protected-authored candidate rejects generic deferred `Hierarchy` insertion. It preserves `World::set_parent_checked`, but a `Commands` producer had no typed checked reparent route. A producer could queue an opaque closure manually, yet that did not provide a named public command or a `DeferredCommandReport` operation for the result.

`Commands::set_parent_checked(child: EntityId, parent: Option<EntityId>)` now queues a checked reparent for **existing entity IDs**. `None` detaches the child. It does not accept an unresolved deferred spawn token. At `World::apply_deferred`, the command invokes the same `World::set_parent_checked` validation, so missing child, missing parent, self-parent, cycle, and static mobility errors keep their existing typed `SceneError` behavior. A rejection is reported as `DeferredCommandOperation::ReparentChecked` with `DeferredCommandTarget::Resolved(child)`; later queued commands continue.

This is an independent ordered command barrier. The queue completes the earlier contiguous structural batch before the checked reparent and then continues with later commands. Earlier successful edits are **not rolled back** if reparent fails; there is no cross-batch atomic transaction promise. The command's own rejection must not mutate topology or World generation. A preceding deferred despawn can therefore remove the requested parent and yield `MissingParent` for this command.

## Evidence and open acceptance

Focused tests exercise valid attach/detach, preceding despawn then `MissingParent` with a later command still published, cycle rejection without topology or generation change, and missing-child target/error reporting. The cycle rejection fixture now uses two authored nodes with distinct local translations, flushes their derived transforms, then snapshots both node records, local and world transforms, generation, and pending-scene state. A fresh WorldStructure subscriber observes zero new facts after the rejected deferred command. `DeferredBarrierObservation`, used to observe the later command, was already present in the parent test module's preimage; this candidate adds only the checked-reparent test module there. The existing `scene::tests::ecs_commands::structural_batches` and `scene::tests::derived_state::checked_parent_chain` suites cover the neighboring structural and direct checked routes. The grouped Release behavior filter is `scene::tests::ecs_commands::checked_reparent`; there is no ignored benchmark in this slice.

The original eight-path candidate completed static Rustfmt, scoped diff, and exact source/predecessor hash checks. The later G06 regression successor has a separate frozen Batch U preimage in `2026-09-28-runtime1020-g06-deferred-rejection-successor-manifest.json`; its focused Rustfmt, exact-hash, final-newline, and trailing-whitespace checks pass. `git diff --check` reports no errors but these three files are untracked, so it does not inspect their contents. Grouped managed validation remains pending. Managed Cargo compilation and behavior tests are pending asynchronous grouped validation. The Runtime62 1K/100K/1M hierarchy, command-apply latency, render-product p99, and regression limits remain open; this slice makes no measured speed claim. G06 is partially covered by the deferred single-command cycle case, including unchanged topology, local/world transforms, World facts, and generation. Other failure modes and single/batch transaction behavior remain open. Atomicity across an entire queued structural sequence is outside this independent-barrier API.
