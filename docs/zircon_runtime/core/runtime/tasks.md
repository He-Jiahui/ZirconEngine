---
related_code:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
implementation_files:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - zircon_runtime/src/core/runtime/mod.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - tools/tests/test_runtime_job_system_audit.py
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/tasks.rs::isolated_runtime_fixtures_share_the_process_task_owner
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::project_asset_manager_uses_the_injected_runtime_io_pool
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Task Helpers

## Purpose

`zircon_runtime::core::runtime::tasks` is the runtime-kernel owner for low-level task execution helpers, scheduling facades, and concrete runtime task pools. This subtree owns `spawn_named_thread(...)`, moved out of the former core-root `channel_util.rs` fragment because thread creation is runtime execution behavior; `JobScheduler`, moved out of the former core-root `job_scheduler.rs` fragment; and the concrete `TaskPool` / `TaskPools` rayon-backed executors formerly stored under `core/tasks`.

Runtime plan 02 M2.2 retired the old `zircon_runtime::core::tasks` namespace. Callers use `core::runtime::tasks` for the namespace owner, or the curated `core` / prelude facades for the public task-pool types and `JobScheduler`.

## Ownership Boundary

The runtime task helper layer may own direct OS thread creation, error mapping into `ZirconError`, runtime scheduling facades over the compute pool, and the concrete rayon-backed pool implementation. It does not define neutral DTOs; those remain in `core::framework::tasks` and `core::framework::channel`.

Callers that need the helper should import `core::runtime::tasks::spawn_named_thread`. The `core` root no longer re-exports `spawn_named_thread`.

## API

`spawn_named_thread(...)` wraps `std::thread::Builder::name(...).spawn(...)`, preserving the previous behavior and converting spawn failures to `ZirconError::ThreadSpawn` with the requested thread name included in the error text.

`JobScheduler` is re-exported from `core::runtime` and the curated `core` root facade so scene ECS and prelude callers can continue to use the stable scheduler type while the physical implementation sits under the runtime task owner. Runtime 11 extends it with `schedule(...) -> JobHandle`, `schedule_after(...) -> JobHandle`, and `wait_all(...)`, while keeping `spawn` as the detached fire-and-forget helper. `JobSchedulerReport` exposes the scheduler's scheduled/completed counts and wait-time counters.

`JobHandle` is a cheap clone over shared completion state. It supports `is_complete()`, `wait()`, and `combine(...)`. Dependency callbacks launch dependent work only when prerequisites are complete, so `schedule_after` does not occupy a worker thread while it waits. `JobScheduler::wait_all(...)` is the scheduler-owned synchronization helper for a set of handles and records the wait against the scheduler diagnostics state. `parallel_for(...)` is the blocking data-parallel slice primitive for callers that need immediate completion on a specific runtime-owned pool.

`TaskPool` implements the framework-neutral `ParallelSliceExecutor` contract through the same `parallel_for` implementation. This lets framework algorithms such as source cubemap mip generation use a caller-supplied runtime pool without importing `core::runtime` or touching Rayon directly. The runtime owner remains responsible for thread allocation and execution; the framework contract never creates a pool.

`TaskPools::default()` is the process-wide execution owner. It initializes exactly one compute/async-compute/IO set through `OnceLock<TaskPools>` and returns cheap clones thereafter. `TaskPoolOptions::create_pools()` bypasses that default only when a caller explicitly requests an isolated owner. `TaskPool::shares_execution_owner_with(...)` makes this ownership contract testable without relying on OS thread counts. The crate-private current-worker query lets executor-owned resources avoid waiting on work queued behind their own single worker.

Scheduler diagnostics are recorded under `tasks.scheduled`, `tasks.completed`, `tasks.dependency_wait_ms`, and `tasks.main_thread_wait_ms`. `JobHandle::wait()` and `JobScheduler::wait_all(...)` both contribute to the explicit main-thread wait counter. `JobScheduler::record_diagnostics(...)` publishes those counters into `DiagnosticStore` using `tasks` and `job_scheduler` tags.

Current production consumers of `spawn_named_thread(...)` include asset event filtering. Asset decode no longer uses this helper: `AssetWorkerPool` submits decode jobs to its injected runtime IO pool and tracks only request lifecycle state.

## Validation

The 2026-06-12 M2.1 migration evidence includes:

- source scans found no remaining `core::channel_util`, `core::types`, root `spawn_named_thread`, root `ChannelSender`, root `ChannelReceiver`, or root `ServiceObject` imports under `zircon_runtime/src`.
- `rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs` passed with 4 tests.
- `cargo check -p zircon_runtime --lib --locked` passed with pre-existing warnings.
- Runtime 11 M1/M3 static slices add handle/dependency/parallel-for, scheduler `wait_all`, diagnostics, worker-side wait assist, dependency-chain, and fanout tests in `zircon_runtime/src/tests/tasks.rs`; Cargo execution is pending a clean validation window because other cargo/rustc lanes were active.
- `job_system_boundary` now provides a static structure mirror for the task owner: `expected_module_count = 9`, `direct_rayon_paths = 2`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 4`, `behavior_test_anchor_count = 13`, `missing_behavior_test_anchors = []`, `oversized_modules = []`, and `risks = []`. The 2026-06-21 `job_system_inventory_split_static_passed_cargo_deferred_tests_deferred` slice moves source/Rayon ownership into `job_system_source_inventory.py` and API/test/doc anchor ownership into `job_system_anchor_inventory.py`, leaving `job_system_boundary.py` as the audit reader and renderer. The behavior anchors now include panic-safe handle completion for scheduled jobs, dependent jobs, and worker-side wait assist through the task-pool-owned `assist_current_thread_once(...)` helper.
- the source-cubemap cutover regression requires exactly the two classified direct-Rayon paths, verifies the neutral executor contract and `TaskPool` implementation anchors, and rejects any Rayon reference in `source_cubemap/mipmap.rs`.
