---
related_code:
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
implementation_files:
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/index.md
tests:
  - rustfmt --edition 2021 zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs zircon_runtime/src/scene/ecs/mod.rs zircon_runtime/src/scene/tests/ecs_schedule.rs zircon_runtime/src/scene/tests/ecs_schedule/conflict_graph.rs zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - schedule_parallel_executor_runs_registered_batches_through_job_scheduler
  - schedule_parallel_executor_can_run_parallel_batches_serially_with_report
  - schedule_parallel_execution_report_records_diagnostic_counts
  - representative_schedule_produces_multi_system_parallel_batches
  - parallel_and_serial_execution_reach_identical_world_state
  - schedule_parallel_report_keeps_run_batches_compatible
  - schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts
  - cargo test -p zircon_runtime --lib ecs_schedule --locked --target-dir E:/cargo-targets/zircon-runtime-03-0612 -- --nocapture --test-threads=1 failed before executing schedule tests on unrelated unresolved import `crate::asset::ui_v2_asset_references` in zircon_runtime/src/ui/tests/asset_dependency_index.rs
doc_type: module-detail
---

# Schedule Parallel Executor

This document records the runtime 03 M3.1 schedule-executor observability slice. The executor still owns only batch execution over already-built `ScheduleParallelBatch` values; conflict detection and batch construction remain in `schedule_conflict_graph.rs`.

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

Source-level validation pins the compatibility wrapper, report fields, disabled serial path, and diagnostic constants in `ecs_schedule_parallel_executor_structure.rs`.

Behavior coverage in `ecs_schedule/parallel_executor.rs` covers:

- default parallel execution reports one parallel batch and one serial single-system batch for the existing read/read/write fixture;
- disabled execution runs the same fixture serially and reports one serial fallback;
- report diagnostics publish the current parallel-batch and serial-fallback values to `CoreRuntime` diagnostics.
- a representative mixed read/write schedule produces 3 two-system batches;
- default parallel execution and disabled serial execution reach the same representative world state for that schedule.

Cargo validation is pending. The focused `ecs_schedule` run compiled until an unrelated UI test import error in `zircon_runtime/src/ui/tests/asset_dependency_index.rs`, so these schedule tests did not execute.
