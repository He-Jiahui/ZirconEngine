---
related_code:
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/framework/tasks/mod.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
plan_sources:
  - user: 2026-05-16 continue Bevy-style app/prelude/state/time/tasks/log/diagnostic completion
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - dev/bevy/crates/bevy_app/src/task_pool_plugin.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - docs/zircon_runtime/core/job_system.md
tests:
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/tests/prelude.rs
  - cargo test -p zircon_runtime --lib tasks --locked
  - rustfmt --edition 2021 --check zircon_runtime\src\core\runtime\tasks\mod.rs zircon_runtime\src\graphics\visibility\mod.rs zircon_runtime\src\ui\component\catalog\material_foundation\form_controls.rs zircon_runtime\src\ui\component\catalog\material_foundation\selection_inputs.rs zircon_runtime\src\ui\component\state_reducer\keyboard.rs zircon_runtime\src\ui\tests\component_catalog\component_state\keyboard.rs zircon_runtime\src\ui\tests\component_catalog\material_foundation\form_controls.rs zircon_runtime\src\ui\tests\component_catalog\material_foundation\selection_inputs.rs (passed after private diagnostics-state import repair)
  - cargo test -p zircon_runtime material_editor_foundation_catalog_covers_planned_component_layers --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-keyboard-routes-0613-coremin --message-format short --color never -- --test-threads=1 --nocapture (passed after private diagnostics-state import repair)
doc_type: module-detail
---

# Runtime Task Pools

## Purpose

`zircon_runtime::core::runtime::tasks` is the concrete runtime executor layer for Bevy-style task pool categories. The framework task module names the shared vocabulary (`Compute`, `AsyncCompute`, and `Io`), while this module owns the actual rayon-backed thread pools used by `CoreRuntime`.

This keeps the Bevy split that matters for engine behavior: frame-critical CPU work goes to `Compute`, multi-frame background work goes to `AsyncCompute`, and blocking or low-duty IO work goes to `Io`. Zircon does not copy Bevy's global singleton model; task pools are owned by each `CoreRuntime` instance and exposed through `CoreRuntime`/`CoreHandle`.

## Reference Evidence

Bevy installs default pools through `TaskPoolPlugin` and allocates IO, async-compute, then compute threads with a percentage policy. Zircon mirrors that allocation strategy in `TaskPoolOptions`: IO defaults to 25% up to four workers, async-compute defaults to 25% up to four workers, and compute receives the remaining workers with at least one worker.

Bevy's `bevy_tasks/src/usages.rs` defines the semantic distinction between `ComputeTaskPool`, `AsyncComputeTaskPool`, and `IoTaskPool`. Zircon carries those semantics through `TaskPoolKind` and runtime-owned `TaskPools`.

## Ownership Boundary

The task contracts under `zircon_runtime::core::framework::tasks` remain pure DTOs and diagnostics contracts. They do not spawn work.

The concrete pools under `zircon_runtime::core::runtime::tasks` own thread creation and execution. `CoreRuntime` initializes one `TaskPools` set at construction time, and `CoreHandle::task_pools()` exposes it to runtime services and managers without requiring global state.

`JobScheduler` is implemented under the same owner because scheduling is runtime execution behavior. It delegates to the compute pool so existing callers can schedule frame-critical work without creating a second thread pool. The curated `core` and prelude facades re-export the task-pool types and `JobScheduler`, but the old `core::tasks` namespace has been retired.

## Data Model

- `TaskPool` wraps one rayon pool plus its `TaskPoolDescriptor`.
- `TaskPools` stores the compute, async-compute, and IO pools for one runtime instance.
- `TaskPoolOptions` configures total thread bounds and per-pool assignment policies.
- `TaskPoolThreadCounts` records the resolved thread distribution for diagnostics and tests.
- `TaskPoolThreadAssignmentPolicy` computes one pool's worker count from remaining and total workers.
- `TaskPoolReport` and `TaskPoolReportEntry` snapshot the runtime-owned pool distribution into stable diagnostic text, including the resolved totals, pool count, configured worker count, actual rayon parallelism, pool kind, and thread-name stem.
- `JobSchedulerReport` snapshots logical scheduler activity: scheduled/completed task counts plus dependency wait and explicit handle-wait time in milliseconds.

The public execution surface now has two layers. The pool layer keeps `spawn` for detached work, `install` for running a closure inside the pool, and `join` for simple fork/join work. The JobSystem layer adds `JobHandle`, `JobScheduler::schedule`, `JobScheduler::schedule_after`, `JobScheduler::wait_all`, `JobHandle::combine`, and `parallel_for(...)`. The dependency scheduler launches dependent work through completion callbacks instead of consuming a worker while waiting on prerequisites; `JobHandle::wait()` remains the handle-level synchronization point, while `JobScheduler::wait_all(...)` is the scheduler-owned multi-handle synchronization point for owner-controlled frame or test boundaries.

`parallel_for(...)` is the first data-parallel primitive. It accepts a runtime-owned `TaskPool`, a mutable slice, a chunk size, and a per-chunk closure. Chunk size `0` is normalized to `1`, and the function blocks until every chunk has been processed. Direct rayon use should move behind this module in Runtime 11 M2 so thread budget remains governed by `TaskPoolOptions`.

## Diagnostics

`TaskPools::report()` is the read-only diagnostic surface for task-pool composition. `CoreRuntime::task_pool_report()` and `CoreHandle::task_pool_report()` expose the same report from the runtime boundary so callers do not need to know where the concrete pools are stored. The report mirrors the shape used by app module-selection diagnostics: `diagnostic_lines()` returns stable key/value lines and `format_diagnostics()` joins them for log files, command-line tooling, or tests. The report does not spawn work and does not expose rayon internals; it only describes the already-created pools.

`JobScheduler::diagnostic_report()` is the read-only diagnostic surface for logical scheduled work. `JobScheduler::record_diagnostics(store, frame)` publishes the same values into `DiagnosticStore` with `tasks` and `job_scheduler` tags. `spawn`, `schedule`, and `schedule_after` increment `tasks.scheduled`; task closures increment `tasks.completed`; dependency-release latency is accumulated in `tasks.dependency_wait_ms`; explicit `JobHandle::wait()` and `JobScheduler::wait_all(...)` synchronization is accumulated in `tasks.main_thread_wait_ms`.

`JobSchedulerDiagnosticsState` remains private to `core::runtime::tasks`. `tasks/mod.rs` imports it for sibling task modules, but does not re-export it outside the owner module. The public diagnostic surface remains `JobSchedulerReport`, the scheduler diagnostic methods, and the stable `tasks.*` diagnostic keys.

The line format starts with aggregate allocation:

- `tasks.total_threads`
- `tasks.io_threads`
- `tasks.async_compute_threads`
- `tasks.compute_threads`
- `tasks.pools`

Each pool then emits one `task_pool.kind=...` line with the actual parallelism, configured worker-thread count, and thread-name stem. This gives dev-profile logging and module diagnostics a Bevy-style answer to which pool owns which work category without depending on Bevy's global singleton `ComputeTaskPool` / `AsyncComputeTaskPool` / `IoTaskPool` model.

## Validation State

During the 2026-06-13 editor UI grouped-keyboard validation, the Material catalog lib-test compile hit Rust `E0365` because `tasks/mod.rs` tried to `pub(super) use diagnostics::JobSchedulerDiagnosticsState` while the diagnostics state is intentionally private. Changing that line to an internal `use` preserved the module owner boundary and unblocked the focused grouped-selection reducer test, the Material catalog coverage test, and the scoped `cargo check -p zircon_runtime --lib --no-default-features --features core-min` run in `E:\cargo-targets\zircon-editor-ui-keyboard-routes-0613-coremin`.

## Test Coverage

`zircon_runtime/src/tests/tasks.rs` verifies default Bevy-style thread distribution, small-host minimum pool availability, execution on all three pools, formatted task-pool diagnostics, runtime/handle report access, and the `JobScheduler` facade relationship to the compute pool. Runtime 11 adds tests for handle waiting, dependency scheduling, combined handles, scheduler `wait_all`, dependency scheduling on a single-worker pool, exact data-parallel visitation, chunk-size granularity, scheduler diagnostics, deep dependency chains, and wide fanout `JobHandle::combine`.

`zircon_runtime/src/tests/prelude.rs` verifies that the stable runtime prelude exports the task-pool types and diagnostic report needed by app and module authors.

`job_system_boundary` mirrors the Runtime 11 structure without Cargo: `expected_module_count = 9`, `direct_rayon_paths = 2`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 4`, `behavior_test_anchor_count = 12`, `missing_behavior_test_anchors = []`, `oversized_modules = []`, and `risks = []`. The behavior anchors now include panic-safe handle completion for scheduled jobs and dependent jobs.
