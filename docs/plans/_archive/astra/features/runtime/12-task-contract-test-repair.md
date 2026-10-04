---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
---

# Task contract test repair

## Repairs

The cancellation token's node is visible only within the task-graph owner so
detached and scoped execution can construct the same token after the module split.
Archive writer blockers use `schedule` to retain completion handles; their release
and wait sequence is unchanged. Reader equivalents are recorded in M14.

The runtime pool regression now checks three independent worker domains and
compares actual scheduler/Compute worker thread identities in a single-worker
Compute configuration. The private scheduler owner comparison stays private.
The event-reader retirement test explicitly types its higher-ranked closure.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M15 | Cancellation token boundary, archive handles, worker-domain and event-reader tests | implemented_pending_validation | Latest run reports 1 lib and 102 lib-test errors; these repairs join the next batch |

Existing cancellation, worker shutdown, archive and event-reader behavioral tests
must pass before acceptance. No performance result is claimed from compilation.
