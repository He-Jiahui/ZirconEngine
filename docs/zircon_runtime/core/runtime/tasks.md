---
related_code:
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/pending.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pool/owned_workers.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_editor/src/core/logging/runtime_task_diagnostics
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
implementation_files:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/pending.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pool/owned_workers.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_editor/src/core/logging/runtime_task_diagnostics
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-17-task-system-static-review.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-17-task-diagnostics-accuracy.md
  - docs/plans/zircon_runtime/runtime/11/2026-07-17-task-diagnostics-accuracy-current-source.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs::detached_spawn_counts_panicked_tasks_as_completed
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation/tests.rs
  - zircon_runtime/src/core/runtime/tasks/execution/runtime.rs::tests::shutdown_joins_owned_workers_even_when_pool_handles_are_retained
  - zircon_runtime/src/core/runtime/tasks/execution/runtime.rs::tests::shutdown_timeout_keeps_unjoined_workers_visible_and_retryable
  - zircon_runtime/src/core/runtime/tasks/execution/runtime.rs::tests::shutdown_from_owned_worker_returns_incomplete_without_self_joining
  - zircon_editor/src/core/logging/runtime_task_diagnostics/tests.rs
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_track_ready_queue_active_and_queue_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_queue_pressure_matrix_drains_without_gauge_leaks
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_reports_conserved_lifecycle_snapshots_during_transitions
  - zircon_runtime/src/tests/tasks.rs::worker_side_wait_is_reported_as_explicit_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_distinguish_panics_from_dependency_cancellation
  - tools/tests/test_frameworks_02_core_error_single_source.py
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

The runtime task helper layer may own direct OS thread creation, error mapping into the canonical `CoreError`, runtime scheduling facades over the compute pool, and the concrete rayon-backed pool implementation. It does not define neutral DTOs; those remain in `core::framework::tasks` and `core::framework::channel`.

Callers that need the helper should import `core::runtime::tasks::spawn_named_thread`. The `core` root no longer re-exports `spawn_named_thread`.

## API

`spawn_named_thread(...)` wraps `std::thread::Builder::name(...).spawn(...)`, returns `CoreResult<JoinHandle<T>>`, and converts spawn failures to `CoreError::ThreadSpawn` with the requested thread name included in the error text. No task-local or compatibility error enum remains.

`JobScheduler` is re-exported from `core::runtime` and the curated `core` root facade so scene ECS and prelude callers can continue to use the stable scheduler type while the physical implementation sits under the runtime task owner. Runtime 11 extends it with `schedule(...) -> JobHandle`, `schedule_after(...) -> JobHandle`, and `wait_all(...)`, while keeping `spawn` as the detached fire-and-forget helper. Detached work records completion through an unwind-safe guard, so a panicking fire-and-forget task cannot leave `tasks.scheduled - tasks.completed` permanently inflated. `JobSchedulerReport` exposes the scheduler's scheduled/completed counts and wait-time counters.

`JobHandle` is a cheap clone over shared completion state. It supports `is_complete()`, `wait()`, and `combine(...)`. Dependency callbacks launch dependent work only when prerequisites are complete, so `schedule_after` does not occupy a worker thread while it waits. `JobScheduler::wait_all(...)` is the scheduler-owned synchronization helper for a set of handles and records the wait against the scheduler diagnostics state. `parallel_for(...)` is the blocking data-parallel slice primitive for callers that need immediate completion on a specific runtime-owned pool.

`TaskPool` implements the framework-neutral `ParallelSliceExecutor` contract through `parallel_for` and the source-order-preserving `parallel_map_indices`. This lets framework algorithms such as source cubemap mip generation and explicitly executor-owned Mesh SDF cooks use a caller-supplied runtime pool without importing `core::runtime` or touching Rayon directly. The runtime owner remains responsible for thread allocation and execution; the framework contract never creates a pool.

`TaskPools::default()` is the process-wide execution owner. It initializes exactly one compute/async-compute/IO set through `OnceLock<TaskPools>` and returns cheap clones thereafter. `TaskPoolOptions::create_pools()` bypasses that default only when a caller explicitly requests an isolated owner. `TaskPool::shares_execution_owner_with(...)` makes this ownership contract testable without relying on OS thread counts. The crate-private current-worker query lets executor-owned resources avoid waiting on work queued behind their own single worker.

An explicit `ExecutionRuntime` adds a stronger lifecycle contract around its isolated `TaskPools`. Its retained pools are the sole strong Rayon owners; cloned `TaskPool` routes hold only a weak backend handle and one shared atomic admission gate. Each call checks the gate around its weak upgrade, so admitted calls finish without a hot-path mutex while post-close handles cannot retain or reopen workers. Runtime shutdown closes all three domains before removing their sole pools. `pool/owned_workers.rs` owns the standard-library join handles created through Rayon's custom spawn hook. After scope drain, `ExecutionRuntime::shutdown(...)` waits for Rayon termination and joins every handle, publishing per-domain expected/exited/joined counts. Timeout keeps the same owners `Closing` for retry. A shutdown call made from an owned worker returns an incomplete receipt for that domain instead of waiting to join itself; an external retry completes the transition after the task returns. This receipt covers only the explicit runtime pools; process-default pools, the timer, and private subsystem workers remain separate migration owners.

Scheduler diagnostics expose `tasks.scheduled` and `tasks.completed`, current `tasks.dependency_waiting` / `tasks.queued` / `tasks.active` gauges, cumulative `tasks.queue_wait_ms` with `tasks.queue_wait_samples`, `tasks.panicked`, `tasks.cancelled`, `tasks.dependency_wait_ms`, and `tasks.explicit_wait_ms`. The lifecycle gauges conserve `scheduled = completed + dependency_waiting + queued + active`. Lifecycle and duration writers remain atomic-only and bracket each update with an in-flight count plus monotonic epoch; overlapping writers retire through an acquire/release chain before the final zero-writer state is published. Readers make at most 16 attempts, accept only a writer-free unchanged-epoch snapshot, and otherwise return the last confirmed stable report-side snapshot; this preserves bounded reader progress and keeps queue-wait duration paired with its sample count without a per-task mutex. A dependent job cancelled before launch moves from dependency-waiting directly to completed/cancelled without entering queued or active state, while detached panic completion is recorded by an unwind-safe guard. `JobScheduler::record_diagnostics(...)` publishes the same snapshot into `DiagnosticStore` using `tasks` and `job_scheduler` tags.

`JobScheduler::task_diagnostic_source()` enables observation-only collection, without activating full lifecycle counter/timing sampling, and exposes runtime-neutral panic/cancellation facts through typed scheduler/task identity and a monotonic cursor. Retention is fixed at 256 entries, each message is capped at 4 KiB on a UTF-8 boundary, and one read returns at most 64 entries with an exact dropped-count when a consumer falls behind. The journal is not a general log store: successful tasks never enter it, and the editor adapter advances its own cursor while emitting through the sole `EditorLogService` runtime source.

The former `tasks.main_thread_wait_ms` surface was a false claim because `JobHandle::wait()` is legal on workers and no caller identity is carried by the handle. Runtime11 hard-cuts that field to `tasks.explicit_wait_ms`; no alias or compatibility counter survives. Main-thread stall attribution, when needed, must be joined with an external thread/frame trace instead of inferred from the scheduler counter.

Current production consumers of `spawn_named_thread(...)` include asset event filtering. Asset decode no longer uses this helper: `AssetWorkerPool` submits decode jobs to its injected runtime IO pool and tracks only request lifecycle state.

## Validation

The 2026-06-12 M2.1 migration evidence includes:

- source scans found no remaining `core::channel_util`, `core::types`, root `spawn_named_thread`, root `ChannelSender`, root `ChannelReceiver`, or root `ServiceObject` imports under `zircon_runtime/src`.
- `rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs` passed with 4 tests.
- `cargo check -p zircon_runtime --lib --locked` passed with pre-existing warnings.
- Runtime 11 M1/M3 static slices add handle/dependency/parallel-for, scheduler `wait_all`, diagnostics, worker-side wait assist, dependency-chain, and fanout tests in `zircon_runtime/src/tests/tasks.rs`; Cargo execution is pending a clean validation window because other cargo/rustc lanes were active.
- `job_system_boundary` now provides a static structure mirror for the task owner: `expected_module_count = 13`, `diagnostic_anchor_count = 11`, `behavior_test_anchor_count = 46`, `missing_behavior_test_anchors = []`, `missing_api_snippets = {}`, `oversized_modules = []`, and `runtime_editor_dependency_references = []`. The current full-tree scan sees three direct-Rayon paths instead of the two task-owner whitelist entries because `graphics/.../mesh_draw_command_list/builder.rs` is still an external migration blocker; the two aggregate `risks` entries describe only that mismatch. The 2026-06-21 inventory split keeps source/Rayon ownership in `job_system_source_inventory.py` and API/test/doc anchor ownership in `job_system_anchor_inventory.py`, with `job_system_boundary.py` as audit reader and risk aggregator. Runtime11's added behavior anchors cover observation-only sampling separation, bounded terminal retention/batches, typed severity/identity, UTF-8 message bounds, shard-local identity uniqueness, dependency-panic classification, and first-terminal-winner consistency.
- the source-cubemap cutover regression requires exactly the two classified direct-Rayon paths, verifies the neutral executor contract and `TaskPool` implementation anchors, and rejects any Rayon reference in `source_cubemap/mipmap.rs`.
