---
related_code:
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/debug.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
plan_sources:
  - user: 2026-06-13 implement runtime architecture plan code
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/index.md
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Task.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Pipe.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskConcurrencyLimiter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/ParallelFor.h
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - dev/bevy/crates/bevy_tasks/src/slice.rs
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
tests:
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - cargo test -p zircon_runtime --lib tasks --locked -- --nocapture
  - cargo test -p zircon_runtime --lib job --locked -- --nocapture
  - cargo test -p zircon_runtime --lib worker_pool --locked -- --nocapture
  - runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass
  - runtime_11_job_system_mirror_docs_match_structure_audit_counts
  - job_system_boundary targeted audit: expected_module_count = 9, direct_rayon_paths = 2, schedule_parallel_executor_direct_rayon = [], diagnostic_anchor_count = 4, behavior_test_anchor_count = 12, missing_behavior_test_anchors = [], oversized_modules = [], mirror_docs_guard_present = true, risks = []
  - runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending static checks passed 2026-06-16
doc_type: module-detail
---

# Runtime Job System

## Scope

Runtime 11 extends the existing Bevy-style task pools into a small JobSystem layer. The owner remains `zircon_runtime::core::runtime::tasks`; framework DTOs stay in `core::framework::tasks`, and consumers reach concrete execution through `CoreRuntime`, `CoreHandle`, `JobScheduler`, or an explicit `TaskPool`.

This document records the M0 model decision before the M1 code surface: `JobHandle`, dependency scheduling, explicit synchronization points, a `parallel_for` primitive, and the first scheduler diagnostics surface. It also records which candidate primitives are intentionally not implemented yet.

The structural mirror is `job_system_boundary` under `runtime_structure_audits/`. Current targeted evidence reports `expected_module_count = 9`, `direct_rayon_paths = 2`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 4`, `behavior_test_anchor_count = 12`, `missing_behavior_test_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_11_job_system_mirror_docs_match_structure_audit_counts` keeps this module doc, Runtime 11, the runtime index, the M0 review, and runtime-interface convergence synchronized with those counts.

## Consumer Matrix

| Consumer | Current path | Required primitive | Decision |
|---|---|---|---|
| ECS parallel batches | `zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs` | batch-local fork/join plus batch dependency chain | Runtime 11 M2.3 now submits batches through `schedule_after` handles and waits only on the tail batch; Runtime 11 M2.2 has also moved batch-local two-through-six joins and generic larger-batch fanout behind `JobScheduler::join(...)`, so the executor no longer imports Rayon directly. |
| Graphics frustum culling | `zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs` | range or slice data parallelism with stable output order | Runtime 11 M2.1 routes large-scene frustum work through the render framework's `compute_task_pool` and `parallel_for(...)`; `parallel_frustum.rs` no longer imports Rayon directly. |
| Asset decode worker | `zircon_runtime/src/asset/pipeline/worker_pool.rs` | IO-lane long work, completion notification, bounded queue semantics | Runtime 11 M2.4 uses explicit accounting: worker threads remain self-managed to preserve Runtime 04 queue/de-dup/completion semantics, while production defaults are derived from `TaskPoolOptions` IO thread counts and recorded through asset worker diagnostics. |
| Runtime module families | animation/navigation/physics/plugin consumers | reusable scheduling handle without direct rayon | Expose `JobHandle` and `schedule_after`; do not add priority/cancellation until a concrete consumer needs it. |
| Future physics fixed step | Runtime 01 physics decision | fixed-step internal parallelism and frame-end sync | Reserve `JobScheduler::wait_all(...)` and `JobHandle::wait` / combined handles as the frame synchronization points; backend-specific thread-pool integration is a later physics decision. |

## Model Selection

| Dimension | Unity semantic anchor | UE5 task anchor | Zircon decision |
|---|---|---|---|
| Handle | `JobHandle` value that can be completed or combined | `FTask` / `FTaskHandle` with `Wait`, `BusyWait`, and completion checks | `JobHandle` is a cheap clone over shared completion state. It supports `is_complete`, `wait`, and `combine`. |
| Dependencies | `Schedule(dependsOn)` and `CombineDependencies` | `Launch(..., Prerequisites(...))` | `JobScheduler::schedule_after(&[JobHandle], task)` launches only after all dependencies complete. The implementation uses completion callbacks instead of occupying a worker while waiting. |
| Sync point | Main-thread `Complete()` | `Wait()` and `BusyWait()` | `JobHandle::wait()` is the handle-level sync point, and `JobScheduler::wait_all(...)` is the scheduler-owned multi-handle sync point. Runtime frame-loop integration is reserved for Runtime 03 follow-up. |
| Serial domain | No dedicated pipe in the core semantic model | `FPipe` FIFO serial pipe | No pipe in M1. Existing consumers can express serial order as dependencies; add a named pipe only if asset or editor workloads produce evidence that dependency chains are insufficient. |
| Data parallelism | `IJobParallelFor` with inner-loop batch count | `ParallelFor` with minimum batch size and worker limits | `parallel_for(pool, items, chunk_size, f)` wraps rayon chunk execution through a runtime-owned `TaskPool`. It is blocking and intended for per-frame CPU transforms such as culling or batch-local ECS work. |
| Concurrency limit | Not part of the minimal semantic surface | `FTaskConcurrencyLimiter` | Not implemented in M1. Runtime 04 backpressure and asset diagnostics are the first valid trigger. |
| Worker wait | Unity discourages worker-side completion waits | Godot and UE both include explicit deadlock avoidance paths | M1 avoids dependency-wait deadlocks by not scheduling dependent work until prerequisites complete. Direct `wait()` from arbitrary worker code is not a new scheduling primitive and should remain a main-thread or owner-controlled sync point until a work-assist design is proven. |

## Thread Budget

`TaskPoolOptions` remains the only runtime thread-budget owner. It declares total thread bounds and distributes workers across compute, async-compute, and IO pools. The plan for remaining bypasses is:

- Direct rayon use moves behind `core::runtime::tasks` primitives. `pool.rs` and `parallel_for.rs` are the allowed task-execution Rayon owners; `runtime_absorption::rayon_boundary` now enforces that boundary for production sources.
- Graphics frustum culling consumed the Runtime 11 M2.1 render-owner window on 2026-06-16. `WgpuRenderFramework` now carries a `compute_task_pool`, runtime module construction supplies `core.task_pools().compute().clone()`, and `VisibilityContext::from_extract_with_history_static_index_and_task_pool(...)` passes that pool into `parallel_frustum.rs`. Current guard status is `runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending`, with `direct_rayon_paths = 2`.
- Asset worker threads use the explicit-accounting route from 11-M2.4. `ProjectAssetManager::default()` builds `AssetWorkerPoolOptions` from `TaskPoolOptions::default().resolve_thread_counts(...).io_threads`; explicit manager construction remains an override and diagnostics publish the resulting `asset.worker.budgeted_threads` path.
- No global rayon pool is introduced. Runtime code should execute through per-runtime pools so `CoreRuntime` remains the execution owner.

## API Contract

`JobScheduler::spawn` remains fire-and-forget. `JobScheduler::schedule` returns a `JobHandle`. `JobScheduler::schedule_after` returns a handle for the dependent task without blocking a worker while dependencies are outstanding. `JobHandle::combine` creates a synchronization handle that completes when all child handles complete. `JobScheduler::wait_all(...)` is the scheduler-owned multi-handle synchronization point; it combines the provided handles and records the explicit wait against the scheduler diagnostics state.

Handle-backed scheduled tasks are panic-safe at the synchronization boundary. If a scheduled task panics, its `JobHandle` still reaches a terminal state, wakes waiters, and `wait()` reports the task panic on the caller thread. `schedule_after` and `JobHandle::combine` propagate dependency panic state to their returned handles without running dependent task bodies, so a failed prerequisite cannot leave a synchronization point waiting forever.

`parallel_for` is blocking and uses an explicit chunk size. A chunk size of zero is normalized to one item per chunk. Callers use it when they need stable completion before continuing the current frame; longer lived work should be scheduled with handles instead.

`ScheduleParallelExecutor` is the first runtime consumer of dependency scheduling. It chains every `ScheduleParallelBatch` from the previous batch handle, records the report counts up front, waits on the final batch handle, and then replays each batch result in source order to keep deterministic error reporting.

## Observability

`JobScheduler` owns a shared diagnostics state across clones. `spawn`, `schedule`, and `schedule_after` increment `tasks.scheduled`; `spawn` increments `tasks.completed` when its fire-and-forget closure returns, while handle-backed jobs increment `tasks.completed` when their returned handle reaches its first terminal state, including panic and dependency-failure terminal states. Dependent jobs record `tasks.dependency_wait_ms` from `schedule_after` submission until dependency release either launches the task or fails it because a prerequisite panicked. `JobHandle::wait()` and `JobScheduler::wait_all(...)` record explicit synchronization cost in `tasks.main_thread_wait_ms`.

`JobScheduler::diagnostic_report()` exposes an in-memory `JobSchedulerReport`; `JobScheduler::record_diagnostics(store, frame)` publishes the same values into `DiagnosticStore` with `tasks` and `job_scheduler` tags.

Asset worker budget accounting remains in the asset diagnostic namespace because the worker pool still owns its request/completion channels. `asset.worker.budgeted_threads` is the bridge metric that lets task-budget analysis count those self-managed IO workers alongside `TaskPoolOptions`.

## Test Coverage

`zircon_runtime/src/tests/tasks.rs` owns the first M1/M3 behavior anchors. `job_system_boundary` now keeps `behavior_test_anchor_count = 12` with `missing_behavior_test_anchors = []` so those names cannot silently drift while Cargo validation is pending. `asset/tests/pipeline/worker_pool.rs` owns the M2.4 budget-accounting anchors:

- `job_handle_wait_blocks_until_task_completes`
- `job_handle_wait_reports_task_panic_without_leaking_completion`
- `schedule_after_runs_task_only_after_all_dependencies`
- `schedule_after_propagates_dependency_panic_without_running_dependent_task`
- `combined_handle_completes_when_all_children_complete`
- `schedule_after_does_not_consume_worker_while_waiting_on_dependencies`
- `job_diagnostics_track_schedule_complete_and_wait_times`
- `deep_dependency_chain_completes_in_order`
- `wide_fanout_combine_waits_for_all`
- `scheduler_wait_all_waits_for_all_handles_and_records_sync_time`
- `parallel_for_visits_every_item_exactly_once`
- `parallel_for_chunk_size_bounds_task_granularity`
- `executor_batches_are_chained_through_job_dependencies`
- `schedule_parallel_batches_chain_through_job_handles`
- `schedule_parallel_executor_does_not_call_rayon_directly`
- `rayon_is_only_reachable_through_core_task_primitives`
- `rayon_render_exception_cutover_is_recorded_in_runtime_11_m2_1_status`
- `worker_pool_options_can_derive_threads_from_runtime_io_budget`
- `project_asset_manager_default_workers_use_runtime_io_budget_source`
- `runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass`

Cargo execution reached package compilation but did not reach the task tests on 2026-06-13: `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture` first hit a plugin native-loader test import error for `PluginInterfaceManifest`. The missing import has been fixed. A 2026-06-20 clean-window rerun of `cargo test -p zircon_runtime --lib tasks --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-11-validation-0620 --message-format short --color never -- --test-threads=1 --nocapture` stayed in `zircon_runtime` lib-test compilation for the 1200s tool window plus an additional 650s wait and produced no test binary or test result; the residual Cargo/rustc processes from that run were stopped. A narrower 2026-06-20 core-min rerun, `cargo test -p zircon_runtime --lib tasks --no-default-features --features core-min --locked --jobs 1 --target-dir E:\Git\ZirconEngine\target\codex-runtime11-coremin-0620 --message-format short --color never -- --test-threads=1 --nocapture`, also timed out after 1200s during `zircon_runtime` lib-test compilation, produced no `zircon_runtime*.exe` test binary in that target directory, and had matching residual Cargo/rustc command lines stopped. The required milestone commands remain recorded in Runtime 11, and these timeout records do not count as Cargo passes.

The 2026-06-20 lightweight guard pass confirms the static boundary while Cargo remains pending: standalone `job_system.rs` passed 1/1, standalone `rayon_boundary.rs` passed 3/3, standalone `asset_worker_policy.rs` passed 1/1, and `asset_worker_policy.rs` passed rustfmt. The asset worker guard was tightened to inspect the `impl AssetWorkerPool` block for the retired `AssetWorkerPool::new(worker_count)` signature while still requiring `AssetWorkerPoolOptions` to own worker-count configuration, so the guard no longer mistakes the valid `AssetWorkerPoolOptions::new(worker_count)` constructor for the retired pool API.

The core-min window added another lightweight evidence pass before status sync: `job_system_boundary.py` compiled, direct `job_system_boundary_audit` reported `expected_module_count = 9`, `direct_rayon_paths = 2`, `behavior_test_anchor_count = 12`, `missing_behavior_test_anchors = []`, and `risks = []`, and standalone `job_system.rs` 1/1 plus standalone `rayon_boundary.rs` 3/3 passed.

`runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass` keeps the `tasks/ecs_schedule/worker_pool/rayon` validation lane visible across Runtime 11, the runtime index, Runtime 05 closeout, this module doc, and the M0 review. The render-owned `parallel_frustum.rs` direct-Rayon cutover is complete at static/source level, but Runtime 11 remains `in_progress` until the declared package filters have real Cargo evidence.
