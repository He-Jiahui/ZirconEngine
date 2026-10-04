---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/ecs/commands/command.rs
  - zircon_runtime/src/scene/ecs/commands/commands/checked_reparent.rs
  - zircon_runtime/src/scene/ecs/commands/commands/facade.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-deferred-checked-reparent-command.md
tests:
  - zircon_runtime/src/scene/tests/ecs_commands/checked_reparent.rs
---

# Runtime1020: deferred checked reparent command

Status: `candidate_static_review_complete_managed_validation_pending`.

- [x] Save exact Batch T successor predecessor hashes, preserve unrelated changes, and claim the eight bounded paths under the shared Session.
- [x] Add a named `Commands::set_parent_checked` route for existing entity IDs, executed as an independent ordered barrier through the validated `World::set_parent_checked` API.
- [x] Report a typed `ReparentChecked` failure against the resolved child while allowing subsequent queued commands to execute.
- [x] Cover valid attach/detach, preceding despawn to missing parent, cycle and missing child rejection, error target, and unchanged topology/generation on command rejection.
- [x] Extend the deferred cycle rejection regression with flushed, nonidentity authored transforms; assert both node records, local/world transforms, pending-scene state, and a fresh WorldStructure subscriber's fact count remain unchanged.
- [x] Preserve the preexisting `DeferredBarrierObservation` test resource; only the checked-reparent module declaration was added to its parent test file.
- [x] Record the independent-barrier and no-cross-batch-rollback contract; keep deferred spawn tokens outside this API.
- [x] Complete scoped Rustfmt, diff, and source-hash seal for this candidate.
- [ ] Obtain grouped managed Cargo compilation and `scene::tests::ecs_commands::checked_reparent` behavior-test acceptance.
- [ ] Complete Runtime62 G06 across the remaining failure modes and single/batch reparent transaction paths. The deferred cycle case now asserts unchanged topology, local/world transforms, World facts, and generation; grouped managed execution is pending.
- [ ] Measure Runtime62 product-scale command and hierarchy latency and close the 1K/100K/1M and render-product p99 gates; no performance threshold is currently proven.

This completion list records a candidate, not accepted product performance. No direct Cargo, coordinator validation submission, tooling edit, commit, push, or external notification was performed in this slice.
