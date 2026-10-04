---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
---

# Regression contract updates

## Repairs

Updated regression callers to the current checked scene-parent API, fallible
entity creation, borrowed reflection path/plugin accessors, owned image-fixture
path and canonical ResourceLocator display text. Host-command tests read desired
state from the command payload; backend Arc cloning permits trait-object coercion.

Fixed two shadowed fixture function names, shader URI borrowing and explicit
u32 benchmark accumulators. The sixteen-parameter conflict regression extracts
its error without requiring an unsupported sixteen-element tuple Debug impl.

Idle runtime shutdown uses a finite deadline and verifies all three physical
worker domains have joined. A zero-duration deadline cannot guarantee thread exit.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M16 | Current regression APIs, benchmark types and physical shutdown assertions | implemented_pending_validation | Derived from M1-M12 terminal diagnostic; include in the next combined run |
