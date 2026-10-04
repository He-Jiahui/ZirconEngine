---
related_code:
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
implementation_files:
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-17-ecs-schedule-static-review.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/index.md
tests:
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs::tests::cloned_task_registry_shares_frozen_task_map_until_mutated
  - rustfmt --edition 2021 zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs zircon_runtime/src/scene/ecs/mod.rs zircon_runtime/src/scene/tests/ecs_schedule.rs zircon_runtime/src/scene/tests/ecs_schedule/conflict_graph.rs zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - schedule_parallel_executor_runs_registered_batches_through_job_scheduler
  - schedule_parallel_executor_can_run_parallel_batches_serially_with_report
  - schedule_parallel_execution_report_records_diagnostic_counts
  - representative_schedule_produces_multi_system_parallel_batches
  - parallel_and_serial_execution_reach_identical_world_state
  - executor_batches_are_chained_through_job_dependencies
  - schedule_parallel_report_keeps_run_batches_compatible
  - schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts
  - schedule_parallel_batches_chain_through_job_handles
  - schedule_parallel_executor_does_not_call_rayon_directly
  - rayon_is_only_reachable_through_core_task_primitives
  - cargo test -p zircon_runtime --lib ecs_schedule --locked --target-dir E:/cargo-targets/zircon-runtime-03-0612 -- --nocapture --test-threads=1 failed before executing schedule tests on unrelated unresolved import `crate::asset::ui_v2_asset_references` in zircon_runtime/src/ui/tests/asset_dependency_index.rs
doc_type: module-detail
---

# Schedule Parallel Executor

This document records the runtime 03 M3 schedule-executor observability slice and the runtime 11 M2.3/M2.2 task-model integration. The executor still owns only batch execution over already-built `ScheduleParallelBatch` values; conflict detection and batch construction remain in `schedule_conflict_graph.rs`.

## Owner Contract

`ScheduleParallelExecutor` now has two execution entry points:

- `run_batches(...)` keeps the previous compatibility surface and discards the report.
- `run_batches_with_report(...)` returns `ScheduleParallelExecutionReport`.

The report exposes these counters:

- `parallel_batches`: number of batches executed through Rayon-backed parallel paths.
- `serial_batches`: number of batches executed serially.
- `serial_fallbacks`: serial batches that had more than one system because parallel execution was disabled.
- `executed_systems`: total systems reached by completed batches.

The executor also exposes `parallel_enabled()` and `with_parallel_enabled(false)`. The default remains parallel execution enabled.

## Batch Dependency Chain

`run_batches_with_report(...)` now submits each batch as a `JobScheduler::schedule_after(...)` task. The first batch depends on a completed handle; every following batch depends on the previous batch handle. The caller waits only on the tail handle before replaying batch results in original batch order.

The task registry stores the complete immutable task map behind `Arc<HashMap<...>>`, so each scheduled batch can move an O(1) snapshot into the runtime task pool without borrowing or copying the full registry. A later `register(...)` uses copy-on-write and therefore cannot mutate a snapshot already owned by scheduled work. Batch-local execution uses `JobScheduler::join(...)` for fixed two-through-six system paths and a balanced recursive `run_parallel_tasks(...)` helper for larger batches. The executor no longer imports or calls Rayon directly; Rayon remains reachable only through the core task primitives.

If a batch returns a missing-task or task-failed error, a shared abort flag is set. Later scheduled batches complete as no-ops, and `run_batches_with_report(...)` returns the first error in batch order. This preserves the previous failure contract while expressing batch order through `JobHandle` dependencies.

## Diagnostics

`ScheduleParallelExecutionReport::record_diagnostics(...)` records two frame-scoped values through the existing core diagnostics channel:

- `schedule.parallel_batches`
- `schedule.serial_fallbacks`

Both use unit `batch` and subsystem tags `schedule` and `parallel`.

The diagnostic write is deliberately report-owned. Callers can decide where a frame boundary exists without making the executor depend on a specific scene or dynamic-session owner.

## Serial Fallback

When parallel execution is disabled, every batch runs through the same task registry and error mapping as the parallel path. Multi-system batches are counted as serial fallbacks. Single-system batches are serial batches but not fallbacks.

This preserves deterministic task lookup and task-failure reporting:

- missing task errors still report the missing system id before running that task;
- task failures still map to the system id at the same batch slot;
- `run_batches(...)` retains the previous `Result<(), ScheduleParallelExecutorError<E>>` contract.

## Validation Status

Source-level validation pins the compatibility wrapper, report fields, disabled serial path, diagnostic constants, JobHandle batch chain, and the no-direct-Rayon executor boundary in `ecs_schedule_parallel_executor_structure.rs`. `runtime_absorption::rayon_boundary` also scans production runtime sources so direct Rayon usage stays limited to `core/runtime/tasks/{pool,parallel_for}.rs`; the previous render-owned `graphics/visibility/culling/parallel_frustum.rs` exception was cut over in Runtime 11 M2.1.

Behavior coverage in `ecs_schedule/parallel_executor.rs` covers:

- default parallel execution reports one parallel batch and one serial single-system batch for the existing read/read/write fixture;
- disabled execution runs the same fixture serially and reports one serial fallback;
- report diagnostics publish the current parallel-batch and serial-fallback values to `CoreRuntime` diagnostics.
- a representative mixed read/write schedule produces 3 two-system batches;
- default parallel execution and disabled serial execution reach the same representative world state for that schedule.
- `executor_batches_are_chained_through_job_dependencies` asserts the second and third batches can only observe earlier batch completion, and that the scheduler reports one scheduled/completed job per batch.

Cargo validation is pending. The focused `ecs_schedule` run compiled until an unrelated UI test import error in `zircon_runtime/src/ui/tests/asset_dependency_index.rs`, so these schedule tests did not execute.

The 2026-07-17 performance slice added the copy-on-write registry snapshot regression. It remains Cargo-validation pending. Per-run abort state, per-batch result slots, cloned system-id vectors, and dependency handles remain explicit measurement candidates; no allocation reduction is claimed for those paths yet.

## Production Runner Relationship

`ScheduleParallelExecutor` is not the current product-scene owner. Production `WorldDriver` delegates to `SceneScheduleRunner`, whose compiled native-system plan uses `SystemParamAccess` conflicts to select worker-safe batches. It takes those native systems out of `World`, executes them through `JobScheduler::join` without sharing `&mut World`, and merges worker-local command queues in stable order. Non-worker-safe and non-native lanes remain on the main thread. Worker callbacks and command application share one unwind boundary, so taken systems are restored before either panic resumes.

`NativeSystemScheduleDiagnostics` records the product path's worker-batch/conflict, ready-delay, utilization, callback-latency, callback/conservative-writer counters, and two temporary-control-buffer values. Production overlap is currently proved by behavior tests rather than a published counter. `temporary_control_buffer_count` counts the systems/timings containers plus the optional command-queue-reference container; `temporary_control_buffer_bytes` is their capacity-byte proxy. These values are observability inputs, not global allocator-call counts or an allocation benchmark result. Cargo and product workload validation remain pending.
