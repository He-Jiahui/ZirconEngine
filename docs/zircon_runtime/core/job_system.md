---
related_code:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_markdown.py
implementation_files:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/debug.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_markdown.py
plan_sources:
  - user: 2026-06-13 implement runtime architecture plan code
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-17-task-diagnostics-accuracy.md
  - docs/plans/zircon_runtime/runtime/11/2026-07-17-task-diagnostics-accuracy-current-source.md
  - docs/plans/performance/01/2026-07-17-task-system-static-review.md
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
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs::tests::job_terminal_observer_runs_once_when_dependency_continuation_unwinds
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_track_ready_queue_active_and_queue_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_queue_pressure_matrix_drains_without_gauge_leaks
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_reports_conserved_lifecycle_snapshots_during_transitions
  - zircon_runtime/src/tests/tasks.rs::worker_side_wait_is_reported_as_explicit_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_distinguish_panics_from_dependency_cancellation
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs::tests::detached_spawn_counts_panicked_tasks_as_completed
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_editor/src/tests/host/manager/runtime_lifecycle.rs::repeated_editor_runtime_fixtures_release_every_runtime_root
  - cargo test -p zircon_runtime --lib tasks --locked -- --nocapture
  - cargo test -p zircon_runtime --lib job --locked -- --nocapture
  - cargo test -p zircon_runtime --lib worker_pool --locked -- --nocapture
  - runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass
  - runtime_11_job_system_mirror_docs_match_structure_audit_counts
  - tools/tests/test_runtime_job_system_audit.py
  - tests/acceptance/runtime-job-system-audit-owner-sync.md
  - job_system_boundary targeted audit: expected_module_count = 10, direct_rayon_paths = 2, schedule_parallel_executor_direct_rayon = [], diagnostic_anchor_count = 11, behavior_test_anchor_count = 27, missing_behavior_test_anchors = [], oversized_modules = [], mirror_docs_guard_present = true, risks = []
  - runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending static checks passed 2026-06-16
doc_type: module-detail
---

# Runtime Job System

Runtime 11 current guard-owner sync (2026-07-10): `job_system_boundary` now reports `expected_guard_file_count = 2`, `missing_guard_files = []`, `mirror_docs_guard_present = true`, and `risks = []` by reading both the route parent `job_system.rs` and the real folder-backed `job_system/mirror_docs.rs` owner. `runtime_11_job_system_mirror_docs_match_structure_audit_counts` remains the aggregate mirror guard. JobSystem production behavior is unchanged; the named `tasks/ecs_schedule/worker_pool/rayon` filters retain historical passing evidence, while the broader full-lib final gate remains pending.

## Scope

Runtime 11 extends the existing Bevy-style task pools into a small JobSystem layer. The owner remains `zircon_runtime::core::runtime::tasks`; framework DTOs stay in `core::framework::tasks`, and consumers reach concrete execution through `CoreRuntime`, `CoreHandle`, `JobScheduler`, or an explicit `TaskPool`.

This document records the M0 model decision before the M1 code surface: `JobHandle`, dependency scheduling, explicit synchronization points, a `parallel_for` primitive, and the first scheduler diagnostics surface. It also records which candidate primitives are intentionally not implemented yet.

The structural mirror is `job_system_boundary` under `runtime_structure_audits/`. Current targeted evidence reports `expected_module_count = 10`, `direct_rayon_paths = 2`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 11`, `behavior_test_anchor_count = 27`, `missing_behavior_test_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. The tenth owner, `timer.rs`, is the process-owned bounded deadline service for lifecycle maintenance; it prevents asset consumers from creating maintenance threads. The 2026-06-21 `job_system_inventory_split_static_passed_cargo_deferred_tests_deferred` slice moved the source/Rayon inventory to `job_system_source_inventory.py` and the declaration/API/test/doc anchors to `job_system_anchor_inventory.py`; the follow-up `job_system_markdown_split_static_passed_cargo_deferred_tests_deferred` slice keeps `job_system_boundary.py` as the audit reader, missing-anchor calculator, and risk aggregator while `job_system_markdown.py` owns the Markdown renderer. The `worker_wait_assist_static_passed_cargo_deferred` slice keeps Rayon work-assist encapsulated in `pool.rs` through `assist_current_thread_once(...)`, while `JobHandle::wait()` uses that helper to avoid single-worker self-deadlock when a runtime worker waits on a child handle. `runtime_11_job_system_mirror_docs_match_structure_audit_counts` keeps this module doc, Runtime 11, the runtime index, the M0 review, and runtime-interface convergence synchronized with those counts.

## Consumer Matrix

| Consumer | Current path | Required primitive | Decision |
|---|---|---|---|
| ECS parallel batches | `zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs` | batch-local fork/join plus batch dependency chain | Runtime 11 M2.3 now submits batches through `schedule_after` handles and waits only on the tail batch; Runtime 11 M2.2 has also moved batch-local two-through-six joins and generic larger-batch fanout behind `JobScheduler::join(...)`, so the executor no longer imports Rayon directly. |
| Graphics frustum culling | `zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs` | range or slice data parallelism with stable output order | Runtime 11 M2.1 routes large-scene frustum work through the render framework's `compute_task_pool` and `parallel_for(...)`; `parallel_frustum.rs` no longer imports Rayon directly. |
| Asset decode worker | `zircon_runtime/src/asset/pipeline/worker_pool.rs` | IO-lane long work, completion notification, bounded queue semantics | Runtime 11 M2.4 submits unique decode requests directly to the injected runtime IO pool. Backpressure, de-duplication, completion fanout, panic terminalization, and Drop waiting remain asset-owned; thread creation does not. |
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
| Worker wait | Unity discourages worker-side completion waits | Godot and UE both include explicit deadlock avoidance paths | M1 avoids dependency-wait deadlocks by not scheduling dependent work until prerequisites complete. Direct `wait()` from arbitrary worker code remains discouraged as a gameplay-facing primitive, but Runtime 11 now has a proven wait-assist fallback: when called from a Rayon worker, `JobHandle::wait()` asks the current pool to execute one pending task before parking briefly. |

## Thread Budget

`TaskPoolOptions` remains the only runtime thread-budget owner. It declares total thread bounds and distributes workers across compute, async-compute, and IO pools. The plan for remaining bypasses is:

- `TaskPools::default()` is the process-wide task owner backed by `OnceLock<TaskPools>`. Default `CoreRuntime` and `ProjectAssetManager` instances clone the same three pool handles; creating 128 isolated runtime states does not construct 128 worker sets. `TaskPoolOptions::create_pools()` is the explicit isolated-owner path.

- Direct rayon use moves behind `core::runtime::tasks` primitives. `pool.rs` and `parallel_for.rs` are the allowed task-execution Rayon owners; `runtime_absorption::rayon_boundary` now enforces that boundary for production sources.
- Source cubemap mip generation consumes the neutral framework `ParallelSliceExecutor` contract. The explicit-executor builders route large-face work through the caller's runtime-owned pool; synchronous builders stay serial because no runtime execution owner was supplied. Neither path creates a hidden pool or falls back to Rayon's process-global pool.
- Graphics frustum culling consumed the Runtime 11 M2.1 render-owner window on 2026-06-16. `WgpuRenderFramework` now carries a `compute_task_pool`, runtime module construction supplies `core.task_pools().compute().clone()`, and `VisibilityContext::from_extract_with_history_static_index_and_task_pool(...)` passes that pool into `parallel_frustum.rs`. Current guard status is `runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending`, with `direct_rayon_paths = 2`.
- Asset decoding uses the IO-pool execution route from 11-M2.4. `ProjectAssetManager::new(io_task_pool)` injects the owner explicitly, while `ProjectAssetManager::default()` uses the process owner. `AssetWorkerPool` has no `zircon-asset-*` thread, worker-count option, explicit budget source, or workerless test constructor.
- Rayon's implicit process-global default pool is not used. Runtime work executes through the process-default `TaskPools` owner or a deliberately constructed isolated `TaskPool`; runtime state and scheduler diagnostics remain local to each `CoreRuntime`.

## API Contract

`JobScheduler::spawn` remains fire-and-forget. `JobScheduler::schedule` returns a `JobHandle`. `JobScheduler::schedule_after` returns a handle for the dependent task without blocking a worker while dependencies are outstanding. `JobHandle::combine` creates a synchronization handle that completes when all child handles complete. `JobScheduler::wait_all(...)` is the scheduler-owned multi-handle synchronization point; it combines the provided handles and records the explicit wait against the scheduler diagnostics state.

`JobHandle::wait()` is deadlock-resistant when invoked from a runtime worker. The handle wait loop drops its state lock, calls the task-pool-owned `assist_current_thread_once(...)`, and only parks briefly when the current Rayon worker finds no ready task. This keeps direct Rayon calls in the existing `pool.rs` owner and prevents a single-worker scheduler from blocking forever while the only worker waits on a child job it just queued.

Handle-backed scheduled tasks are panic-safe at the synchronization boundary. If a scheduled task panics, its `JobHandle` still reaches a terminal state, wakes waiters, and `wait()` reports the task panic on the caller thread. `schedule_after` and `JobHandle::combine` propagate dependency panic state to their returned handles without running dependent task bodies, so a failed prerequisite cannot leave a synchronization point waiting forever.

`JobHandle::on_terminal(...)` registers a general one-shot observer for successful, panicked, or dependency-cancelled terminal state. Registration before terminal state stores the observer. Registration while dependency continuations are being published joins the queued observer set; registration after continuation publication invokes it inline before returning. Terminal publication releases existing dependency continuations in their original order and only then delivers queued observers outside the job-state lock. `wait()` synchronizes terminal state rather than observer completion, so observers must remain bounded and own any stronger consumer completion signal. Observer panic is contained, increments only `JobHandle::terminal_observer_panic_count()`, and does not rewrite the task panic or dependency result.

The observer is deliberately application-neutral. Runtime 11 does not import winit, the dynamic API, or host cadence policy, and it does not wake the scheduler or event loop for every completed job. A subsystem that owns frame-visible asynchronous output may attach an observer and route its own session-scoped wake; invisible jobs attach none.

`parallel_for` is blocking and uses an explicit chunk size. A chunk size of zero is normalized to one item per chunk. `TaskPool` also implements the framework-neutral `ParallelSliceExecutor` contract through the same implementation, allowing framework algorithms to request slice parallelism without depending on the runtime task module. Callers use it when they need stable completion before continuing the current frame; longer lived work should be scheduled with handles instead.

`ScheduleParallelExecutor` is the first runtime consumer of dependency scheduling. It chains every `ScheduleParallelBatch` from the previous batch handle, records the report counts up front, waits on the final batch handle, and then replays each batch result in source order to keep deterministic error reporting.

## Observability

`JobScheduler` diagnostics are off by default; callers that need lifecycle telemetry construct the scheduler with `with_diagnostics()` before submitting work. The enabled scheduler clones share a fixed 64-shard diagnostics state. A submitting or worker thread receives one cache-aligned shard, so lifecycle updates do not contend on a scheduler-global writer counter. Each shard retains the acquire/release retirement chain and a bounded 16-attempt stable read. Frame reporting then verifies the epoch and retirement state of the full shard set after merging, with a separately bounded aggregate retry; it publishes one complete aggregate snapshot under a single short cache lock, or returns the preceding complete aggregate while writers continue to mutate. Work admitted while diagnostics are off remains untracked even if collection is enabled later, so a terminal event can never appear without its matching admission. `tasks.dependency_waiting`, `tasks.queued`, `tasks.active`, and `tasks.completed` are derived from the merged counters and conserve `tasks.scheduled`; cumulative `tasks.queue_wait_ms` is paired with the same stable started count exposed as `tasks.queue_wait_samples`. A dependent closure prevented from launching by a prerequisite panic increments `tasks.cancelled` without entering enqueued/started state. Detached work uses an unwind-safe completion guard, so its terminal event remains accurate while Rayon retains ownership of panic handling.

`tasks.dependency_wait_ms` remains the separate submission-to-dependency-release duration for `schedule_after`. `JobHandle::wait()` and `JobScheduler::wait_all(...)` now record `tasks.explicit_wait_ms`. The previous `tasks.main_thread_wait_ms` name was removed rather than aliased because the handle can be waited from any thread and the scheduler has no authoritative caller-thread identity; consumers must not infer a main-thread stall from an explicit synchronization duration.

`JobScheduler::diagnostic_report()` exposes an in-memory `JobSchedulerReport`; `JobScheduler::record_diagnostics(store, frame)` publishes the same values into `DiagnosticStore` with `tasks` and `job_scheduler` tags.

Asset request accounting remains in the asset diagnostic namespace because the orchestration layer still owns admission, de-duplication, completion fanout, and frame deltas. `asset.worker.budgeted_threads` mirrors the shared IO pool's parallelism for correlation; it must not be added to `TaskPoolOptions` totals as another allocation.

## Test Coverage

`zircon_runtime/src/tests/tasks.rs` and the private `job_handle.rs` test module own the M1/M3 behavior anchors. `job_system_boundary` now keeps `behavior_test_anchor_count = 27` with `missing_behavior_test_anchors = []` so terminal-observer races and unwind delivery, dependency-waiting release/cancellation, the queue/active pressure matrix, conserved lifecycle snapshots, worker explicit wait, panic/cancellation, combined-handle barrier completion, and prior dependency/parallel tests cannot silently drift while Cargo validation is pending. Continuations are individually contained so one unwind cannot prevent later scheduler/combine callbacks; observers run after every continuation, then the first continuation panic is rethrown. The detached-panic helper test is additionally anchored in `job_scheduler.rs`. `asset/tests/pipeline/worker_pool.rs` owns the M2.4 budget-accounting anchors:

- `job_handle_wait_blocks_until_task_completes`
- `job_handle_wait_reports_task_panic_without_leaking_completion`
- `schedule_after_runs_task_only_after_all_dependencies`
- `schedule_after_propagates_dependency_panic_without_running_dependent_task`
- `combined_handle_completes_when_all_children_complete`
- `combined_handle_waits_for_all_children_before_propagating_panic`
- `schedule_after_does_not_consume_worker_while_waiting_on_dependencies`
- `worker_thread_wait_does_not_deadlock_scheduler`
- `job_terminal_observer_registered_before_completion_runs_once`
- `job_terminal_observer_registered_after_completion_runs_once`
- `multiple_job_terminal_observers_each_run_exactly_once`
- `job_terminal_observer_panic_is_contained_and_recorded`
- `job_terminal_observer_preserves_dependency_continuation_order`
- `job_terminal_observer_can_reenter_handle_accessors`
- `job_terminal_observer_runs_once_when_dependency_continuation_unwinds`
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
- `isolated_runtime_fixtures_share_the_process_task_owner`
- `explicit_task_pool_options_create_an_isolated_task_owner`
- `project_asset_manager_uses_the_injected_runtime_io_pool`
- `project_asset_manager_defaults_share_the_process_io_pool`
- `dropping_worker_pool_waits_for_its_runtime_io_jobs`
- `dropping_worker_pool_on_its_io_worker_does_not_deadlock_pending_jobs`
- `repeated_editor_runtime_fixtures_release_every_runtime_root`
- `runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass`

Cargo execution reached package compilation but did not reach the task tests on 2026-06-13: `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture` first hit a plugin native-loader test import error for `PluginInterfaceManifest`. The missing import has been fixed. A 2026-06-20 clean-window rerun of `cargo test -p zircon_runtime --lib tasks --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-11-validation-0620 --message-format short --color never -- --test-threads=1 --nocapture` stayed in `zircon_runtime` lib-test compilation for the 1200s tool window plus an additional 650s wait and produced no test binary or test result; the residual Cargo/rustc processes from that run were stopped. A narrower 2026-06-20 core-min rerun, `cargo test -p zircon_runtime --lib tasks --no-default-features --features core-min --locked --jobs 1 --target-dir E:\Git\ZirconEngine\target\codex-runtime11-coremin-0620 --message-format short --color never -- --test-threads=1 --nocapture`, also timed out after 1200s during `zircon_runtime` lib-test compilation, produced no `zircon_runtime*.exe` test binary in that target directory, and had matching residual Cargo/rustc command lines stopped. The required milestone commands remain recorded in Runtime 11, and these timeout records do not count as Cargo passes.

The 2026-06-20 lightweight guard pass confirms the static boundary while Cargo remains pending: standalone `job_system.rs` passed 1/1, standalone `rayon_boundary.rs` passed 3/3, standalone `asset_worker_policy.rs` passed 1/1, and `asset_worker_policy.rs` passed rustfmt. The asset worker guard was tightened to inspect the `impl AssetWorkerPool` block for the retired `AssetWorkerPool::new(worker_count)` signature while still requiring `AssetWorkerPoolOptions` to own worker-count configuration, so the guard no longer mistakes the valid `AssetWorkerPoolOptions::new(worker_count)` constructor for the retired pool API.

The core-min window added another lightweight evidence pass before status sync: `job_system_boundary.py` compiled, direct `job_system_boundary_audit` reported `expected_module_count = 9`, `direct_rayon_paths = 2`, `behavior_test_anchor_count = 12`, `missing_behavior_test_anchors = []`, and `risks = []`, and standalone `job_system.rs` 1/1 plus standalone `rayon_boundary.rs` 3/3 passed.

The 2026-06-21 inventory split compiled `job_system_boundary.py`, `job_system_source_inventory.py`, and `job_system_anchor_inventory.py`; direct `job_system_boundary_audit` continued to report task owner modules 9/9, direct Rayon paths 2/2, diagnostic anchors 4/4, behavior-test anchors 12/12, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. The follow-up Markdown renderer split also compiled `job_system_markdown.py`, moved `render_job_system_boundary_markdown` out of `job_system_boundary.py`, and left the direct audit counts unchanged at `risks = []`.

The 2026-06-21 worker wait-assist slice adds `worker_thread_wait_does_not_deadlock_scheduler`, bringing Runtime 11 behavior-test anchors to 13/13. `pool.rs` remains one of the two direct-Rayon owners and now exposes `assist_current_thread_once(...)`; `job_handle.rs` uses that helper plus `WORKER_WAIT_IDLE_PARK` to avoid self-deadlock without adding another Rayon owner path. Standalone `job_system.rs` 1/1, standalone `rayon_boundary.rs` 3/3, and standalone `plan_status.rs` 33/33 remain the lightweight guards for this lane until package-level `tasks/ecs_schedule/worker_pool/rayon` Cargo gates can run.

`runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass` keeps the `tasks/ecs_schedule/worker_pool/rayon` validation lane visible across Runtime 11, the runtime index, Runtime 05 closeout, this module doc, and the M0 review. The render-owned `parallel_frustum.rs` direct-Rayon cutover is complete at static/source level, but Runtime 11 remains `in_progress` until the declared package filters have real Cargo evidence.
