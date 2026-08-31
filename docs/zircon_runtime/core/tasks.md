---
related_code:
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/pending.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_editor/src/core/logging/runtime_task_diagnostics
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/pending.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - zircon_editor/src/core/logging/runtime_task_diagnostics
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
plan_sources:
  - user: 2026-05-16 continue Bevy-style app/prelude/state/time/tasks/log/diagnostic completion
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-17-task-diagnostics-accuracy.md
  - docs/plans/zircon_runtime/runtime/11/2026-07-17-task-diagnostics-accuracy-current-source.md
  - docs/plans/performance/01/2026-07-17-task-system-static-review.md
  - dev/bevy/crates/bevy_app/src/task_pool_plugin.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - docs/zircon_runtime/core/job_system.md
tests:
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_track_ready_queue_active_and_queue_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_queue_pressure_matrix_drains_without_gauge_leaks
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_reports_conserved_lifecycle_snapshots_during_transitions
  - zircon_runtime/src/tests/tasks.rs::worker_side_wait_is_reported_as_explicit_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_distinguish_panics_from_dependency_cancellation
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs::detached_spawn_counts_panicked_tasks_as_completed
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation/tests.rs
  - zircon_editor/src/core/logging/runtime_task_diagnostics/tests.rs
  - zircon_editor/src/tests/host/manager/runtime_lifecycle.rs::repeated_editor_runtime_fixtures_release_every_runtime_root
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs::tests::job_handle_accessors_recover_poisoned_state_lock
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs::tests::job_handle_wait_recovers_poisoned_state_lock
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs::pending_scheduled_job_recovers_poisoned_task_lock
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles
  - zircon_runtime/src/tests/prelude.rs
  - cargo test -p zircon_runtime --lib tasks --locked
  - rustfmt --edition 2021 --check zircon_runtime\src\core\runtime\tasks\mod.rs zircon_runtime\src\graphics\visibility\mod.rs zircon_runtime\src\ui\component\catalog\material_foundation\form_controls.rs zircon_runtime\src\ui\component\catalog\material_foundation\selection_inputs.rs zircon_runtime\src\ui\component\state_reducer\keyboard.rs zircon_runtime\src\ui\tests\component_catalog\component_state\keyboard.rs zircon_runtime\src\ui\tests\component_catalog\material_foundation\form_controls.rs zircon_runtime\src\ui\tests\component_catalog\material_foundation\selection_inputs.rs (passed after private diagnostics-state import repair)
  - cargo test -p zircon_runtime material_editor_foundation_catalog_covers_planned_component_layers --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-keyboard-routes-0613-coremin --message-format short --color never -- --test-threads=1 --nocapture (passed after private diagnostics-state import repair)
doc_type: module-detail
---

# Runtime Task Pools

## Purpose

`zircon_runtime::core::runtime::tasks` is the concrete runtime executor layer for Bevy-style task pool categories. The framework task module names the shared vocabulary (`Compute`, `AsyncCompute`, and `Io`), while this module owns the actual rayon-backed thread pools used by `CoreRuntime`.

This keeps the Bevy split that matters for engine behavior: frame-critical CPU work goes to `Compute`, multi-frame background work goes to `AsyncCompute`, and blocking or low-duty IO work goes to `Io`. Default runtimes share one process-wide `TaskPools` owner through `OnceLock`; runtime state and scheduler diagnostics remain instance-local and are exposed through `CoreRuntime`/`CoreHandle`.

## Reference Evidence

Bevy installs default pools through `TaskPoolPlugin` and allocates IO, async-compute, then compute threads with a percentage policy. Zircon mirrors that allocation strategy in `TaskPoolOptions`: IO defaults to 25% up to four workers, async-compute defaults to 25% up to four workers, and compute receives the remaining workers with at least one worker.

Bevy's `bevy_tasks/src/usages.rs` defines the semantic distinction between `ComputeTaskPool`, `AsyncComputeTaskPool`, and `IoTaskPool`. Zircon carries those semantics through `TaskPoolKind` and runtime-owned `TaskPools`.

## Ownership Boundary

The task contracts under `zircon_runtime::core::framework::tasks` remain pure DTOs and diagnostics contracts. They do not spawn work.

The concrete pools under `zircon_runtime::core::runtime::tasks` own thread creation and execution. `TaskPools::default()` clones the process-wide owner, so repeated `CoreRuntime::new()` calls do not multiply worker sets. `TaskPoolOptions::create_pools()` remains the explicit isolated-owner path. `CoreHandle::task_pools()` exposes the selected owner to runtime services and managers.

`JobScheduler` is implemented under the same owner because scheduling is runtime execution behavior. It delegates to the compute pool so existing callers can schedule frame-critical work without creating a second thread pool. The curated `core` and prelude facades re-export the task-pool types and `JobScheduler`, but the old `core::tasks` namespace has been retired.

## Data Model

- `TaskPool` wraps one rayon pool plus its `TaskPoolDescriptor`.
- `TaskPools` stores compute, async-compute, and IO pool handles. Its default is process-wide; explicitly constructed values are isolated owners.
- `TaskPoolOptions` configures total thread bounds and per-pool assignment policies.
- `TaskPoolThreadCounts` records the resolved thread distribution for diagnostics and tests.
- `TaskPoolThreadAssignmentPolicy` computes one pool's worker count from remaining and total workers.
- `TaskPoolReport` and `TaskPoolReportEntry` snapshot the runtime-owned pool distribution into stable diagnostic text, including the resolved totals, pool count, configured worker count, actual rayon parallelism, pool kind, and thread-name stem.
- `JobSchedulerReport` snapshots logical scheduler activity: scheduled/completed task counts plus dependency wait and explicit handle-wait time in milliseconds.

The public execution surface now has two layers. The pool layer keeps `spawn` for detached work, `install` for running a closure inside the pool, and `join` for simple fork/join work. The JobSystem layer adds `JobHandle`, `JobScheduler::schedule`, `JobScheduler::schedule_after`, `JobScheduler::wait_all`, `JobHandle::combine`, and `parallel_for(...)`. The dependency scheduler launches dependent work through completion callbacks instead of consuming a worker while waiting on prerequisites; `JobHandle::wait()` remains the handle-level synchronization point, while `JobScheduler::wait_all(...)` is the scheduler-owned multi-handle synchronization point for owner-controlled frame or test boundaries.

### Runtime 15 M3 core runtime task lock poison recovery

Runtime 15 M3 extends the E9/F2 poison-safe lock rule to JobSystem state. `job_handle.rs` now centralizes job-state locking in `JobState::lock_inner()` and condvar wake paths in `wait_inner(...)` / `wait_inner_timeout(...)`, so handle wait, dependent callbacks, terminal marking, panic-message reads, and dependency decrement recover poisoned job state locks. `job_scheduler.rs` centralizes pending scheduled task access in `PendingScheduledJob::lock_task()`, so dependency-release launch and terminal cleanup recover poisoned pending-task locks. The public `JobHandle` / `JobScheduler` API, dependency scheduling model, panic propagation behavior, and diagnostics counters are unchanged.

`parallel_for(...)` is the first data-parallel primitive. It accepts a runtime-owned `TaskPool`, a mutable slice, a chunk size, and a per-chunk closure. Chunk size `0` is normalized to `1`, and the function blocks until every chunk has been processed. Direct rayon use should move behind this module in Runtime 11 M2 so thread budget remains governed by `TaskPoolOptions`.

## Diagnostics

`TaskPools::report()` is the read-only diagnostic surface for task-pool composition. `CoreRuntime::task_pool_report()` and `CoreHandle::task_pool_report()` expose the same report from the runtime boundary so callers do not need to know where the concrete pools are stored. The report mirrors the shape used by app module-selection diagnostics: `diagnostic_lines()` returns stable key/value lines and `format_diagnostics()` joins them for log files, command-line tooling, or tests. The report does not spawn work and does not expose rayon internals; it only describes the already-created pools.

`JobScheduler::diagnostic_report()` is the read-only diagnostic surface for logical scheduled work. `JobScheduler::record_diagnostics(store, frame)` publishes the same values into `DiagnosticStore` with `tasks` and `job_scheduler` tags. Monotonic lifecycle and duration atomics are bracketed by an in-flight writer count and epoch; overlapping writers retire through an acquire/release chain before readers can observe zero active writers. A reader makes at most 16 attempts, derives dependency-waiting, ready-queue, active-task, and terminal gauges (`tasks.dependency_waiting`, `tasks.queued`, `tasks.active`, `tasks.completed`) only from a writer-free unchanged-epoch snapshot, and falls back to its last confirmed stable snapshot under continuous mutation. These gauges conserve `tasks.scheduled`. This keeps reporting bounded and pairs cumulative enqueue-to-start duration/sample count (`tasks.queue_wait_ms`, `tasks.queue_wait_samples`) without putting a mutex on task updates. The report also exposes `tasks.panicked`, `tasks.cancelled`, and dependency-release duration (`tasks.dependency_wait_ms`). `JobHandle::wait()` and `JobScheduler::wait_all(...)` accumulate `tasks.explicit_wait_ms`; no `tasks.main_thread_wait_ms` alias remains because worker-side waits are valid and caller identity is not part of the handle contract.

`JobSchedulerDiagnosticsState` remains private to `core::runtime::tasks`. `tasks/mod.rs` imports it for sibling task modules, but does not re-export it outside the owner module. The public diagnostic surface remains `JobSchedulerReport`, the scheduler diagnostic methods, and the stable `tasks.*` diagnostic keys.

`JobScheduler::task_diagnostic_source()` is the bounded terminal-observation surface for host integration. Its independent observation flag does not activate full lifecycle counter/timing sampling. It retains at most 256 panic/cancellation facts, caps each UTF-8 message at 4 KiB, returns at most 64 observations per cursor read, and reports an exact retention gap. Task identity allocation reuses the 64 diagnostics shards instead of adding a scheduler-global hot atomic. The runtime owns no editor types or log store; the editor host owns its cursor and emits observations through the sole `EditorLogService` as the existing runtime source.

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

`zircon_runtime/src/tests/tasks.rs` verifies default Bevy-style thread distribution, small-host minimum pool availability, execution on all three pools, formatted task-pool diagnostics, runtime/handle report access, and the `JobScheduler` facade relationship to the compute pool. `isolated_runtime_fixtures_share_the_process_task_owner` locks the 128-runtime sharing contract, while `explicit_task_pool_options_create_an_isolated_task_owner` preserves deliberate isolation. Editor's `repeated_editor_runtime_fixtures_release_every_runtime_root` exercises 128 manager-activated runtime fixtures and proves their state roots still close while the process task owner remains shared. Runtime 11 also covers handle waiting, dependency scheduling, worker-side wait assist, combined handles, scheduler `wait_all`, exact data-parallel visitation, chunk-size granularity, scheduler diagnostics, deep dependency chains, and wide fanout `JobHandle::combine`.

`zircon_runtime/src/tests/prelude.rs` verifies that the stable runtime prelude exports the task-pool types and diagnostic report needed by app and module authors.

`job_handle_accessors_recover_poisoned_state_lock`, `job_handle_wait_recovers_poisoned_state_lock`, and `pending_scheduled_job_recovers_poisoned_task_lock` deliberately poison JobHandle state and pending scheduled task locks, then verify dependent callbacks, wait, completion marking, and scheduled task launch still work. `structure_convention/lock_poison_policy.rs::runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles` keeps the task owner files, this document, Runtime 15 status rows, and plan mirrors synchronized under `runtime_15_core_runtime_task_lock_poison_recovery_static_passed_cargo_deferred`.

`job_system_boundary` mirrors the Runtime 11 structure without Cargo: `expected_module_count = 13`, `diagnostic_anchor_count = 11`, `behavior_test_anchor_count = 46`, `missing_behavior_test_anchors = []`, `missing_api_snippets = {}`, `oversized_modules = []`, and `runtime_editor_dependency_references = []`. The current full-tree scan reports three direct-Rayon paths because `graphics/.../mesh_draw_command_list/builder.rs` remains outside the two-path task-owner whitelist; the aggregate audit's two `risks` entries are limited to that external migration blocker. The 2026-06-21 inventory split keeps module/Rayon source ownership in `job_system_source_inventory.py` and API/test/doc anchor ownership in `job_system_anchor_inventory.py`, with the boundary file as audit reader and risk aggregator. Runtime11's seven added anchors cover observation-only sampling separation, bounded retention/batches, severity/identity, UTF-8 bounds, shard-local identity allocation, dependency-panic classification, and first-terminal-winner consistency.
