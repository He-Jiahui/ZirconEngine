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
  - job_system_boundary targeted audit: expected_module_count = 9, direct_rayon_paths = 3, schedule_parallel_executor_direct_rayon = [], diagnostic_anchor_count = 4, oversized_modules = [], mirror_docs_guard_present = true, risks = []
  - pre_m2_1_rayon_render_exception_guard_static_passed_pending_render_owner static checks passed 2026-06-13
doc_type: module-detail
---

# Runtime Job System

## Scope

Runtime 11 extends the existing Bevy-style task pools into a small JobSystem layer. The owner remains `zircon_runtime::core::runtime::tasks`; framework DTOs stay in `core::framework::tasks`, and consumers reach concrete execution through `CoreRuntime`, `CoreHandle`, `JobScheduler`, or an explicit `TaskPool`.

This document records the M0 model decision before the M1 code surface: `JobHandle`, dependency scheduling, explicit synchronization points, a `parallel_for` primitive, and the first scheduler diagnostics surface. It also records which candidate primitives are intentionally not implemented yet.

The structural mirror is `job_system_boundary` under `runtime_structure_audits/`. Current targeted evidence reports `expected_module_count = 9`, `direct_rayon_paths = 3`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 4`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_11_job_system_mirror_docs_match_structure_audit_counts` keeps this module doc, Runtime 11, the runtime index, the M0 review, and runtime-interface convergence synchronized with those counts.

## Consumer Matrix

| Consumer | Current path | Required primitive | Decision |
|---|---|---|---|
| ECS parallel batches | `zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs` | batch-local fork/join plus batch dependency chain | Runtime 11 M2.3 now submits batches through `schedule_after` handles and waits only on the tail batch; Runtime 11 M2.2 has also moved batch-local two-through-six joins and generic larger-batch fanout behind `JobScheduler::join(...)`, so the executor no longer imports Rayon directly. |
| Graphics frustum culling | `zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs` | range or slice data parallelism with stable output order | Add blocking `parallel_for` over mutable slices in M1. Do not edit graphics while render sessions own that area. |
| Asset decode worker | `zircon_runtime/src/asset/pipeline/worker_pool.rs` | IO-lane long work, completion notification, bounded queue semantics | Runtime 11 M2.4 uses explicit accounting: worker threads remain self-managed to preserve Runtime 04 queue/de-dup/completion semantics, while production defaults are derived from `TaskPoolOptions` IO thread counts and recorded through asset worker diagnostics. |
| Runtime module families | animation/navigation/physics/plugin consumers | reusable scheduling handle without direct rayon | Expose `JobHandle` and `schedule_after`; do not add priority/cancellation until a concrete consumer needs it. |
| Future physics fixed step | Runtime 01 physics decision | fixed-step internal parallelism and frame-end sync | Reserve `JobHandle::wait` / combined handles as the frame synchronization point; backend-specific thread-pool integration is a later physics decision. |

## Model Selection

| Dimension | Unity semantic anchor | UE5 task anchor | Zircon decision |
|---|---|---|---|
| Handle | `JobHandle` value that can be completed or combined | `FTask` / `FTaskHandle` with `Wait`, `BusyWait`, and completion checks | `JobHandle` is a cheap clone over shared completion state. It supports `is_complete`, `wait`, and `combine`. |
| Dependencies | `Schedule(dependsOn)` and `CombineDependencies` | `Launch(..., Prerequisites(...))` | `JobScheduler::schedule_after(&[JobHandle], task)` launches only after all dependencies complete. The implementation uses completion callbacks instead of occupying a worker while waiting. |
| Sync point | Main-thread `Complete()` | `Wait()` and `BusyWait()` | `JobHandle::wait()` is the explicit sync point. Runtime frame-loop integration is reserved for Runtime 03/11-M2 and M3. |
| Serial domain | No dedicated pipe in the core semantic model | `FPipe` FIFO serial pipe | No pipe in M1. Existing consumers can express serial order as dependencies; add a named pipe only if asset or editor workloads produce evidence that dependency chains are insufficient. |
| Data parallelism | `IJobParallelFor` with inner-loop batch count | `ParallelFor` with minimum batch size and worker limits | `parallel_for(pool, items, chunk_size, f)` wraps rayon chunk execution through a runtime-owned `TaskPool`. It is blocking and intended for per-frame CPU transforms such as culling or batch-local ECS work. |
| Concurrency limit | Not part of the minimal semantic surface | `FTaskConcurrencyLimiter` | Not implemented in M1. Runtime 04 backpressure and asset diagnostics are the first valid trigger. |
| Worker wait | Unity discourages worker-side completion waits | Godot and UE both include explicit deadlock avoidance paths | M1 avoids dependency-wait deadlocks by not scheduling dependent work until prerequisites complete. Direct `wait()` from arbitrary worker code is not a new scheduling primitive and should remain a main-thread or owner-controlled sync point until a work-assist design is proven. |

## Thread Budget

`TaskPoolOptions` remains the only runtime thread-budget owner. It declares total thread bounds and distributes workers across compute, async-compute, and IO pools. The plan for remaining bypasses is:

- Direct rayon use moves behind `core::runtime::tasks` primitives. `pool.rs` and `parallel_for.rs` are the allowed task-execution Rayon owners; `runtime_absorption::rayon_boundary` now enforces that boundary for production sources.
- Graphics frustum culling is the only tracked exception and should consume `parallel_for` when the render owner window is available. Current guard status is `pre_m2_1_rayon_render_exception_guard_static_passed_pending_render_owner`: `parallel_frustum.rs` is classified as `render-owner-pending-runtime-11-m2-1-cutover`, and actual graphics cutover not executed.
- Asset worker threads use the explicit-accounting route from 11-M2.4. `ProjectAssetManager::default()` builds `AssetWorkerPoolOptions` from `TaskPoolOptions::default().resolve_thread_counts(...).io_threads`; explicit manager construction remains an override and diagnostics publish the resulting `asset.worker.budgeted_threads` path.
- No global rayon pool is introduced. Runtime code should execute through per-runtime pools so `CoreRuntime` remains the execution owner.

## API Contract

`JobScheduler::spawn` remains fire-and-forget. `JobScheduler::schedule` returns a `JobHandle`. `JobScheduler::schedule_after` returns a handle for the dependent task without blocking a worker while dependencies are outstanding. `JobHandle::combine` creates a synchronization handle that completes when all child handles complete.

`parallel_for` is blocking and uses an explicit chunk size. A chunk size of zero is normalized to one item per chunk. Callers use it when they need stable completion before continuing the current frame; longer lived work should be scheduled with handles instead.

`ScheduleParallelExecutor` is the first runtime consumer of dependency scheduling. It chains every `ScheduleParallelBatch` from the previous batch handle, records the report counts up front, waits on the final batch handle, and then replays each batch result in source order to keep deterministic error reporting.

## Observability

`JobScheduler` owns a shared diagnostics state across clones. `spawn`, `schedule`, and `schedule_after` increment `tasks.scheduled`; task closures increment `tasks.completed` when they finish. Dependent jobs record `tasks.dependency_wait_ms` from `schedule_after` submission until dependency release launches the task. `JobHandle::wait()` records explicit synchronization cost in `tasks.main_thread_wait_ms`.

`JobScheduler::diagnostic_report()` exposes an in-memory `JobSchedulerReport`; `JobScheduler::record_diagnostics(store, frame)` publishes the same values into `DiagnosticStore` with `tasks` and `job_scheduler` tags.

Asset worker budget accounting remains in the asset diagnostic namespace because the worker pool still owns its request/completion channels. `asset.worker.budgeted_threads` is the bridge metric that lets task-budget analysis count those self-managed IO workers alongside `TaskPoolOptions`.

## Test Coverage

`zircon_runtime/src/tests/tasks.rs` owns the first M1/M3 behavior anchors, and `asset/tests/pipeline/worker_pool.rs` owns the M2.4 budget-accounting anchors:

- `job_handle_wait_blocks_until_task_completes`
- `schedule_after_runs_task_only_after_all_dependencies`
- `combined_handle_completes_when_all_children_complete`
- `schedule_after_does_not_consume_worker_while_waiting_on_dependencies`
- `job_diagnostics_track_schedule_complete_and_wait_times`
- `deep_dependency_chain_completes_in_order`
- `wide_fanout_combine_waits_for_all`
- `parallel_for_visits_every_item_exactly_once`
- `parallel_for_chunk_size_bounds_task_granularity`
- `executor_batches_are_chained_through_job_dependencies`
- `schedule_parallel_batches_chain_through_job_handles`
- `schedule_parallel_executor_does_not_call_rayon_directly`
- `rayon_is_only_reachable_through_core_task_primitives_or_tracked_render_exception`
- `rayon_render_exception_is_bound_to_runtime_11_m2_1_status`
- `worker_pool_options_can_derive_threads_from_runtime_io_budget`
- `project_asset_manager_default_workers_use_runtime_io_budget_source`
- `runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass`

Cargo execution reached package compilation but did not reach the task tests on 2026-06-13: `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture` first hit a plugin native-loader test import error for `PluginInterfaceManifest`. The missing import has been fixed; the task-test rerun is pending the next clear cargo/rustc window. The required milestone commands are recorded in Runtime 11.

`runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass` keeps the `tasks/ecs_schedule/worker_pool/rayon` validation lane visible across Runtime 11, the runtime index, Runtime 05 closeout, this module doc, and the M0 review. It also keeps the render-owned `parallel_frustum.rs` exception visible until that direct-Rayon cutover has an owner-safe window.
