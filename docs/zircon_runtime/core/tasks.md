---
related_code:
  - zircon_runtime/src/core/tasks/mod.rs
  - zircon_runtime/src/core/tasks/pool.rs
  - zircon_runtime/src/core/tasks/pools.rs
  - zircon_runtime/src/core/tasks/report.rs
  - zircon_runtime/src/core/tasks/thread_assignment.rs
  - zircon_runtime/src/core/job_scheduler.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/framework/tasks/mod.rs
implementation_files:
  - zircon_runtime/src/core/tasks/mod.rs
  - zircon_runtime/src/core/tasks/pool.rs
  - zircon_runtime/src/core/tasks/pools.rs
  - zircon_runtime/src/core/tasks/report.rs
  - zircon_runtime/src/core/tasks/thread_assignment.rs
  - zircon_runtime/src/core/job_scheduler.rs
plan_sources:
  - user: 2026-05-16 continue Bevy-style app/prelude/state/time/tasks/log/diagnostic completion
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_app/src/task_pool_plugin.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
tests:
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/tests/prelude.rs
  - cargo test -p zircon_runtime --lib tasks --locked
doc_type: module-detail
---

# Runtime Task Pools

## Purpose

`zircon_runtime::core::tasks` is the concrete runtime executor layer for Bevy-style task pool categories. The framework task module names the shared vocabulary (`Compute`, `AsyncCompute`, and `Io`), while this module owns the actual rayon-backed thread pools used by `CoreRuntime`.

This keeps the Bevy split that matters for engine behavior: frame-critical CPU work goes to `Compute`, multi-frame background work goes to `AsyncCompute`, and blocking or low-duty IO work goes to `Io`. Zircon does not copy Bevy's global singleton model; task pools are owned by each `CoreRuntime` instance and exposed through `CoreRuntime`/`CoreHandle`.

## Reference Evidence

Bevy installs default pools through `TaskPoolPlugin` and allocates IO, async-compute, then compute threads with a percentage policy. Zircon mirrors that allocation strategy in `TaskPoolOptions`: IO defaults to 25% up to four workers, async-compute defaults to 25% up to four workers, and compute receives the remaining workers with at least one worker.

Bevy's `bevy_tasks/src/usages.rs` defines the semantic distinction between `ComputeTaskPool`, `AsyncComputeTaskPool`, and `IoTaskPool`. Zircon carries those semantics through `TaskPoolKind` and runtime-owned `TaskPools`.

## Ownership Boundary

The task contracts under `zircon_runtime::core::framework::tasks` remain pure DTOs and diagnostics contracts. They do not spawn work.

The concrete pools under `zircon_runtime::core::tasks` own thread creation and execution. `CoreRuntime` initializes one `TaskPools` set at construction time, and `CoreHandle::task_pools()` exposes it to runtime services and managers without requiring global state.

`JobScheduler` remains as a compatibility facade for existing code. It delegates to the compute pool so older callers still schedule frame-critical work without creating a second thread pool.

## Data Model

- `TaskPool` wraps one rayon pool plus its `TaskPoolDescriptor`.
- `TaskPools` stores the compute, async-compute, and IO pools for one runtime instance.
- `TaskPoolOptions` configures total thread bounds and per-pool assignment policies.
- `TaskPoolThreadCounts` records the resolved thread distribution for diagnostics and tests.
- `TaskPoolThreadAssignmentPolicy` computes one pool's worker count from remaining and total workers.
- `TaskPoolReport` and `TaskPoolReportEntry` snapshot the runtime-owned pool distribution into stable diagnostic text, including the resolved totals, pool count, configured worker count, actual rayon parallelism, pool kind, and thread-name stem.

The public execution surface is intentionally narrow: `spawn` for detached background work, `install` for running a closure inside the pool, and `join` for simple fork/join work. More advanced async task handles should build on the framework task descriptors instead of exposing rayon internals.

## Diagnostics

`TaskPools::report()` is the read-only diagnostic surface for task-pool composition. `CoreRuntime::task_pool_report()` and `CoreHandle::task_pool_report()` expose the same report from the runtime boundary so callers do not need to know where the concrete pools are stored. The report mirrors the shape used by app module-selection diagnostics: `diagnostic_lines()` returns stable key/value lines and `format_diagnostics()` joins them for log files, command-line tooling, or tests. The report does not spawn work and does not expose rayon internals; it only describes the already-created pools.

The line format starts with aggregate allocation:

- `tasks.total_threads`
- `tasks.io_threads`
- `tasks.async_compute_threads`
- `tasks.compute_threads`
- `tasks.pools`

Each pool then emits one `task_pool.kind=...` line with the actual parallelism, configured worker-thread count, and thread-name stem. This gives dev-profile logging and module diagnostics a Bevy-style answer to which pool owns which work category without depending on Bevy's global singleton `ComputeTaskPool` / `AsyncComputeTaskPool` / `IoTaskPool` model.

## Test Coverage

`zircon_runtime/src/tests/tasks.rs` verifies default Bevy-style thread distribution, small-host minimum pool availability, execution on all three pools, formatted task-pool diagnostics, runtime/handle report access, and the `JobScheduler` compatibility relationship to the compute pool.

`zircon_runtime/src/tests/prelude.rs` verifies that the stable runtime prelude exports the task-pool types and diagnostic report needed by app and module authors.
