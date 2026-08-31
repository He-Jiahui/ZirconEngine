---
related_code:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle/tests.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/pending.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs
  - zircon_runtime/src/core/runtime/tasks/task_cancellation_policy.rs
  - zircon_runtime/src/core/runtime/tasks/task_descriptor.rs
  - zircon_runtime/src/core/runtime/tasks/task_id.rs
  - zircon_runtime/src/core/runtime/tasks/task_pool_descriptor.rs
  - zircon_runtime/src/core/runtime/tasks/task_pool_kind.rs
  - zircon_runtime/src/core/runtime/tasks/task_state.rs
  - zircon_runtime/src/core/runtime/tasks/task_status.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics/tests.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/mod.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/admission.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/lease.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/options.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope_model.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/task_handle.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope/tests.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/shutdown.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/worker_inventory.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/retained_byte_budget.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/mod.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/key.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pool/owned_workers.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
  - zircon_runtime/src/core/runtime/tasks/timer/tests.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/world/project_io/document.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/src/test_support.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/platform/service_types/driver.rs
  - zircon_runtime/src/platform/preferences/persistence/adapter.rs
  - zircon_runtime/src/platform/test_support.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/platform/preferences/persistence/adapter.rs
  - zircon_runtime/src/platform/tests/preferences.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/settings/persistence.rs
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/core/logging/runtime_task_diagnostics
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
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
  - zircon_runtime/src/core/runtime/tasks/job_handle/tests.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/pending.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics/tests.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/mod.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/admission.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/lease.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/options.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope_model.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope/tests.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/shutdown.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/worker_inventory.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pool/owned_workers.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/world/project_io/document.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
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
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/settings/persistence.rs
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/core/logging/runtime_task_diagnostics
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
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
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/World.cpp
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - dev/bevy/crates/bevy_tasks/src/slice.rs
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
tests:
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs::tests::job_terminal_observer_runs_once_when_dependency_continuation_unwinds
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_track_ready_queue_active_and_queue_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_queue_pressure_matrix_drains_without_gauge_leaks
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_reports_conserved_lifecycle_snapshots_during_transitions
  - zircon_runtime/src/tests/tasks.rs::worker_side_wait_is_reported_as_explicit_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_distinguish_panics_from_dependency_cancellation
  - zircon_runtime/src/core/runtime/tasks/job_scheduler/tests.rs::detached_spawn_counts_panicked_tasks_as_completed
  - zircon_runtime/src/core/runtime/tasks/diagnostics/tests.rs::terminal_observation_source_does_not_enable_full_lifecycle_sampling
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation/tests.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs::tests::shutdown_joins_owned_workers_even_when_pool_handles_are_retained
  - zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs::tests::shutdown_timeout_keeps_unjoined_workers_visible_and_retryable
  - zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs::tests::shutdown_from_owned_worker_returns_incomplete_without_self_joining
  - zircon_runtime/src/scene/module/level_manager_project_io.rs::tests::standalone_level_manager_rejects_artifact_io_without_an_implicit_process_owner
  - zircon_editor/src/core/logging/runtime_task_diagnostics/tests.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_editor/src/tests/host/manager/runtime_lifecycle.rs::repeated_editor_runtime_fixtures_release_every_runtime_root
  - cargo test -p zircon_runtime --lib tasks --locked -- --nocapture
  - cargo test -p zircon_runtime --lib job --locked -- --nocapture
  - cargo test -p zircon_runtime --lib worker_pool --locked -- --nocapture
  - runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass
  - runtime_11_job_system_mirror_docs_match_structure_audit_counts
  - tools/tests/test_runtime_job_system_audit.py
  - tests/acceptance/runtime-job-system-audit-owner-sync.md
  - 2026-07 historical job_system_boundary audit: expected_module_count = 10, direct_rayon_paths = 2, schedule_parallel_executor_direct_rayon = [], diagnostic_anchor_count = 11, behavior_test_anchor_count = 27, missing_behavior_test_anchors = [], oversized_modules = [], mirror_docs_guard_present = true, risks = []
  - runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending static checks passed 2026-06-16
doc_type: module-detail
---

# Runtime Job System

Runtime 11 historical guard-owner snapshot (2026-07-10): `job_system_boundary` reported `expected_guard_file_count = 2`, `missing_guard_files = []`, `mirror_docs_guard_present = true`, and `risks = []` by reading both the route parent `job_system.rs` and the real folder-backed `job_system/mirror_docs.rs` owner. `runtime_11_job_system_mirror_docs_match_structure_audit_counts` remains the aggregate mirror guard. JobSystem production behavior is unchanged; the named `tasks/ecs_schedule/worker_pool/rayon` filters retain historical passing evidence, while the broader full-lib final gate remains pending.

## Scope

Runtime 11 extends the existing task pools into a small JobSystem layer. The
owner is `zircon_runtime::core::runtime::tasks`; task identity, descriptors,
  status, cancellation, logical workload classification, executable handles,
  and shutdown remain
with that owner. `core::framework::tasks` exposes only
`ParallelSliceExecutor`. Consumers reach concrete execution through
`CoreRuntime`, `CoreHandle`, `TaskHandle`, `JobScheduler`, an explicit
`TaskPool`, or an explicit `EngineTaskGraph`.

The 2026-08-28 canonical task-model hard cut deletes the framework
`AsyncTask*` DTO family and unused `TaskPollBudget`. `TaskId`,
`TaskDescriptor`, `TaskState`, and `TaskStatus` are Runtime-owned contracts.
`TaskGraphScope::submit`, `schedule`, and `schedule_after` all return the same
`TaskHandle`; scoped prerequisites are `TaskHandle` values and are lowered to
private `JobHandle` fences only inside the scheduler owner. The handle binds
descriptor, lifecycle snapshot, cancellation, completion, wait, and terminal
observation. `JobHandle` remains the low-level scheduler fence for unscoped
jobs and internal dependency delivery, not a second scoped task identity.
`TaskDescriptor.kind` is a logical class for the single shared TaskGraph; it is
not a physical pool selector. The hard cut names the field accordingly instead
of retaining the misleading legacy `pool` name.

2026-08-26 historical Runtime02 M1 slice registered the bounded `tasks/callback_dispatcher.rs` and `tasks/task_graph/` owners, bringing the JobSystem inventory to twelve modules (`expected_module_count = 12`). `EngineTaskGraph::try_new(...)` owns a deliberately non-static pool set, while `TaskGraphScope` owns admission, cooperative cancellation, per-task status, and queue/running census through its shutdown drain. Releasing the final external `TaskGraphScope` handle closes admission even if task handles still retain internal scope state. `TaskGraphScope::schedule(...)` and `schedule_after(...)` add handle-backed work only after proving that the supplied scheduler uses the matching runtime-owned pool. A prerequisite panic retires the queued scope record as `Failed`; prerequisite cancellation propagates a typed `Cancelled` handle and retires the record as cancelled without running user work. Running work becomes cancelled only after `TaskCancellationToken::acknowledge_cancellation()` confirms that it observed the request; a normal return that ignored a request remains `Completed`. Dynamic-session scene reload preparation is the first production consumer and acknowledges observed scope cancellation. `TaskGraphWorkerInventory` reports that runtime's three pool domains and conserved worker total only; it deliberately excludes the legacy process-default pools, timer, and private worker owners. Scope shutdown currently proves task-body terminal state only, not scheduler-wrapper exit or worker termination, so it is not a dynamic-library unload receipt. Scene target staging, the generic scheduler, process timer, private workers, worker-owner hard cutover, and join receipt remain implementation-pending; the dynamic-library unload P0 and aggregate JobSystem audit remain open.

2026-08-27 superseded three-domain source slice: the explicit `EngineTaskGraph` now closes all three pool domains after scope quiescence, removes the sole Rayon pool owner, and joins custom-spawned worker handles before publishing `Stopped`. Retained `TaskPool` or `JobScheduler` handles carry only a weak backend route plus the shared atomic admission gate, so they cannot retain or reopen the owner after shutdown. `TaskGraphShutdownReport::worker_shutdown` exposes expected/exited/joined counts and termination signal state per domain; a timeout remains `Closing` and can be retried. This is source implementation pending managed Cargo and product profiling. Remaining process-default consumers, the process timer, private workers, and scene target staging remain open, so the aggregate DLL-unload P0 is not accepted.

The superseded 2026-08-27 three-domain production-owner pass injected the activating Runtime pool into `AssetModule`, `PlatformModule`, and editor settings persistence. Asset decode and both preference/settings persistence lanes no longer acquire the process pool through their production factories. `EditorContextBuilder` requires distinct compute and settings-I/O schedulers, while `JobScheduler::from_pool(...)` lets the cross-crate editor derive the latter from its active `CoreHandle`. `DefaultLevelManager::with_core` already gives the scene module the same ownership behavior. `DefaultLevelManager::default()` is memory-only and rejects artifact I/O without a Runtime task owner; standalone tools must own any process-lifetime executor explicitly. Native-plugin discovery is a separate application-process authority because discovery precedes Core composition; it now exposes prepared-root nonblocking refresh and immutable last-good snapshot contracts. Its physical process-pool topology remains profiling-gated rather than receiving a stale per-Runtime route.

Renderer product text remains an explicit exception pending measurement. Product SDF and shape prewarm currently use process Compute, while bitmap raster copies the AsyncCompute count into an additional private OS-thread pool. Offline Font SDF baking is no longer part of that exception: its library API consumes a caller-supplied pool and its standalone CLI explicitly owns and shuts down one TaskGraph. The planned product convergence preserves text-local bounded admission and scratch reuse but moves physical execution to Runtime owners only after the recorded Windows product profile proves the bottleneck and validates the replacement.

WGPU framework construction no longer owns execution topology. Every production-visible constructor accepts an explicit `TaskPool`, module-host graphics and the PBR viewer pass their active `CoreRuntime` TaskGraph worker pool, and the unused implicit Solari constructor is absent. Unit-only `new_for_test*` helpers reuse the CoreRuntime already retained by `ProjectAssetManagerAccess::for_test`; integration and plugin fixtures retain their CoreRuntime alongside the framework. The structure audit rejects graphics-side pool or TaskGraph construction, so adding a new convenience constructor cannot silently restore a full-CPU worker owner.

This document records the M0 model decision before the M1 code surface: `JobHandle`, dependency scheduling, explicit synchronization points, a `parallel_for` primitive, and the first scheduler diagnostics surface. It also records which candidate primitives are intentionally not implemented yet.

The structural mirror is `job_system_boundary` under `runtime_structure_audits/`. The 2026-08-28 current inventory declares `expected_module_count = 22`, `expected_guard_file_count = 2`, `diagnostic_anchor_count = 11`, and `behavior_test_anchor_count = 73`. The seven added owners are the canonical Runtime task contracts. Six current behavior anchors bind descriptor/status/wait to one handle, prove terminal status and completion share one monotonic synchronized state under concurrent observation, require detached cancellation and panic to agree with the dependency fence, retain `CancelOnDrop` prerequisite leases through launch, and permit late terminal observation after the TaskGraph worker set joins. Runtime11 also owns terminal task observation in `tasks/diagnostic_observation/`, reusable bounded blocking-stream capture in `tasks/bounded_stream_io/`, clone-safe retained-result reservations in `tasks/retained_byte_budget.rs`, and bounded per-key serialization in `tasks/bounded_keyed_io/`. The source audit reports no missing modules, declarations, public anchors, behavior anchors, oversized owner, or runtime-to-editor dependency. This is source implementation evidence, not Runtime11 acceptance: managed Cargo, Editor14 migration, product performance/power evidence, and aggregate DLL-unload acceptance remain open. `runtime_11_job_system_mirror_docs_match_structure_audit_counts` keeps this module doc, Runtime 11, the runtime index, the M0 review, and runtime-interface convergence synchronized with those counts.

### Current Single-Owner Cut

The current 2026-08-27 hard cut makes `EngineTaskGraph` the owner of exactly
one physical `zircon-taskgraph-worker` set under one exact global budget.
`TaskGraphScope` retains cancellation and drain accounting, while
`TaskGraphWorkerInventory` reports one set rather than three work-kind domains.
CoreRuntime no longer exports `task_pools()` or `task_pool(kind)`; its scheduler
and the migrated asset, platform, graphics, scene, VM discovery, dynamic
archive, PBR viewer, and editor settings consumers all share
`task_graph().worker_pool()`.

Dynamic session shutdown now drains its session scope, shuts down modules while
scheduling is still available, and only then closes and joins the TaskGraph.
The design, source-model worker counts, and profile gate are recorded in
`docs/plans/optimize/zircon_runtime/11/2026-08-27-engine-task-graph-shared-worker-owner.md`.
Affinity, priority, quotas, keyed-I/O parallelism, timer/process/private-owner
convergence, and managed performance/power evidence remain pending.

## Consumer Matrix

| Consumer | Current path | Required primitive | Decision |
|---|---|---|---|
| ECS parallel batches | `zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs` | batch-local fork/join plus batch dependency chain | Runtime 11 M2.3 now submits batches through `schedule_after` handles and waits only on the tail batch; Runtime 11 M2.2 has also moved batch-local two-through-six joins and generic larger-batch fanout behind `JobScheduler::join(...)`, so the executor no longer imports Rayon directly. |
| Graphics frustum culling | `zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs` | range or slice data parallelism with stable output order | Runtime 11 M2.1 routes large-scene frustum work through the render framework's `compute_task_pool` and `parallel_for(...)`; `parallel_frustum.rs` no longer imports Rayon directly. |
| Asset decode worker | `zircon_runtime/src/asset/pipeline/worker_pool.rs` | I/O-class long work, completion notification, bounded queue semantics | Runtime 11 M2.4 submits unique decode requests through the injected TaskGraph worker owner. The production `AssetModule` constructs `ProjectAssetManager` from the activating Core owner instead of the process default. Backpressure, de-duplication, completion fanout, panic terminalization, and Drop waiting remain asset-owned; thread creation does not. |
| Platform preference persistence | `zircon_runtime/src/platform/preferences/persistence/adapter.rs` | bounded serial I/O lane with runtime-owned worker lifetime | The production `PlatformModule` and the App host-backend descriptor override both inject the active Core TaskGraph owner into `PlatformDriver`; module cleanup drains the lane before TaskGraph shutdown. Neither `PlatformDriver` nor `PreferencePersistenceAdapter` can select a process owner, and tests retain an explicit isolated owner. Deadline registrations remain on the process timer until the measured deadline-owner redesign. |
| Editor settings persistence | `zircon_editor/src/core/settings/persistence.rs` | bounded keyed I/O lane with editor-context shutdown and Runtime-owned workers | `EditorManager` derives an explicit scheduler from the active Core TaskGraph worker owner. Builder and service constructors require that route; editor production source contains no `process_io()` lookup. |
| Dynamic session archive I/O | `zircon_runtime/src/scene/dynamic_scene/session/io/{reader,writer}.rs` | active physical-path single flight, retained result budget, atomic publication, Runtime-owned workers | Reader and writer accept project-resolved physical paths and use `ResolvedProjectPathIdentity` as the typed lane key. Only nonterminal reads share a ticket; terminal retry/refresh creates a new request. Write staging precedes per-path/lineage final-publication gates, and the process registry weakly retains only live path state. Filesystem work runs through the shared TaskGraph worker owner with I/O descriptor semantics. The synchronous session facade and active cooperative cancellation remain open. |
| Native-plugin discovery | `zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs` | process-lifetime discovery authority; product-generation dynamic library host; nonblocking prepared-root publication | Source contract implemented. Editor12 migration and managed validation remain pending. Profile duplicate process/Core pool materialization before selecting an application execution owner or dedicated discovery I/O owner. |
| Offline Font SDF bake | `zircon_runtime/src/text/font_sdf_build_tool/bake.rs` | deterministic batch generation with standalone-process lifetime | `bake_font_sdf_artifact(...)` requires `&TaskPool`; `zircon_font_sdf_bake` explicitly owns one `EngineTaskGraph` and requires bounded shutdown before artifact publication. The library cannot select or initialize process-default pools. |
| Runtime module families | animation/navigation/physics/plugin consumers | reusable scheduling handle without direct rayon | Expose canonical `TaskHandle` for descriptor-led scoped work and prerequisites; keep `JobHandle` as the lower-level unscoped scheduler fence. |
| Future physics fixed step | Runtime 01 physics decision | fixed-step internal parallelism and frame-end sync | Reserve `JobScheduler::wait_all(...)` and `JobHandle::wait` / combined handles as the frame synchronization points; backend-specific thread-pool integration is a later physics decision. |

## Model Selection

| Dimension | Unity semantic anchor | UE5 task anchor | Zircon decision |
|---|---|---|---|
| Handle | `JobHandle` value that can be completed or combined | `FTask` / `FTaskHandle` with `Wait`, `BusyWait`, and completion checks | `TaskHandle` is the canonical admitted-task handle; `JobHandle` remains the low-level unscoped fence/combine primitive. |
| Dependencies | `Schedule(dependsOn)` and `CombineDependencies` | `Launch(..., Prerequisites(...))` | `TaskGraphScope::schedule_after(&[TaskHandle], ...)` launches scoped work only after all canonical prerequisites complete; `JobScheduler::schedule_after(&[JobHandle], ...)` remains the lower-level fence API. |
| Sync point | Main-thread `Complete()` | `Wait()` and `BusyWait()` | `TaskHandle::wait()` synchronizes admitted task lifecycle; `JobHandle::wait()` and `JobScheduler::wait_all(...)` synchronize lower-level unscoped fences. |
| Serial domain | No dedicated pipe in the core semantic model | `FPipe` FIFO serial pipe | No pipe in M1. Existing consumers can express serial order as dependencies; add a named pipe only if asset or editor workloads produce evidence that dependency chains are insufficient. |
| Data parallelism | `IJobParallelFor` with inner-loop batch count | `ParallelFor` with minimum batch size and worker limits | `parallel_for(pool, items, chunk_size, f)` wraps rayon chunk execution through a runtime-owned `TaskPool`. It is blocking and intended for per-frame CPU transforms such as culling or batch-local ECS work. |
| Concurrency limit | Not part of the minimal semantic surface | `FTaskConcurrencyLimiter` | Not implemented in M1. Runtime 04 backpressure and asset diagnostics are the first valid trigger. |
| Worker wait | Unity discourages worker-side completion waits | Godot and UE both include explicit deadlock avoidance paths | M1 avoids dependency-wait deadlocks by not scheduling dependent work until prerequisites complete. Direct `wait()` from arbitrary worker code remains discouraged as a gameplay-facing primitive, but Runtime 11 now has a proven wait-assist fallback: when called from a Rayon worker, `JobHandle::wait()` asks the current pool to execute one pending task before parking briefly. |

## Thread Budget

`EngineTaskGraphOptions` is the CoreRuntime thread-budget owner. It creates one
shared worker set with exactly the configured 1/2/N workers; task kind no
longer selects a physical pool. The plan for remaining bypasses is:

- `TaskPools::default()` is a legacy process-wide owner backed by
  `OnceLock<TaskPools>`. It is not reachable through CoreRuntime and remains
  only for explicitly standalone/process-lifetime paths pending measured
  convergence.

- Direct Rayon use is behind `core::runtime::tasks` primitives. `pool.rs` and `parallel_for.rs` are the only allowed task-execution Rayon owners; the 2026-08-27 `test_runtime_job_system_audit.py` pass is 4/4 and reports zero unclassified production consumers or implicit offline Font SDF owner routes.
- Source cubemap mip generation consumes the neutral framework `ParallelSliceExecutor` contract. The explicit-executor builders route large-face work through the caller's runtime-owned pool; synchronous builders stay serial because no runtime execution owner was supplied. Neither path creates a hidden pool or falls back to Rayon's process-global pool.
- Mesh SDF cooking consumes `parallel_map_indices`; graph encoder buckets consume the Core task owner's ordered index map; mesh command preparation consumes the neutral owned `parallel_map_ordered`. The owned map moves each prepared plan exactly once, preserves input order, and keeps empty/single inputs off the parallel iterator path. Mesh source sorting, owner-thread cache access, and ordered merge behavior remain unchanged.
- Graphics frustum culling consumes the render framework's injected pool, and runtime module construction supplies `core.task_graph().worker_pool().clone()`. `VisibilityContext::from_extract_with_history_static_index_and_task_pool(...)` passes that owner into `parallel_frustum.rs`; `direct_rayon_paths = 2` remains the low-level whitelist.
- Asset decoding uses the worker route injected by `ProjectAssetManager::new(...)`. The production AssetModule supplies the Core TaskGraph owner; `ProjectAssetManager::default()` remains an explicit standalone/process-owner constructor. `AssetWorkerPool` has no `zircon-asset-*` thread or second worker-count option.
- Rayon's implicit global default pool is not used. CoreRuntime work executes through its explicitly owned single-set `EngineTaskGraph`; legacy `TaskPools` construction remains a tracked non-Core migration target.

## API Contract

`JobScheduler::from_pool(...)` is the only public scheduler constructor. It creates a facade over an existing explicit pool without allocating another worker budget; `Default` and `process_io()` are deliberately absent so scheduler construction cannot silently create or select an execution owner. Runtime hosts inject the `EngineTaskGraph` pool, while standalone/process-lifetime tools and tests must name their owner explicitly. `JobScheduler::spawn` remains fire-and-forget. `JobScheduler::schedule` returns a `JobHandle`. `JobScheduler::schedule_after` returns a handle for the dependent task without blocking a worker while dependencies are outstanding. A cancelled dependency produces a cancelled dependent handle without running its body; a panicked dependency retains failure propagation. `JobHandle::terminal_state()` exposes `Completed`, `Failed`, or `Cancelled`, `is_cancelled()` is the focused query, and `wait()` returns normally for cancellation while continuing to report panics. `JobHandle::combine` creates a synchronization handle that completes when all child handles complete and preserves failure over cancellation. `JobScheduler::wait_all(...)` is the scheduler-owned multi-handle synchronization point; it combines the provided handles and records the explicit wait against the scheduler diagnostics state.

`TaskHandle` is the only public identity/status/cancellation handle for admitted
TaskGraph work. It exposes the immutable descriptor, a copied `TaskStatus`
snapshot from the executor-owned record, completion and cancellation queries,
cooperative cancellation request, wait, and terminal observer registration.
The status has no poll counter: consumer observation frequency is not task
lifecycle state. A dynamic-scene task that runs without a scope still uses the
same Runtime record through the crate-owned detached admission path.

`WgpuRenderFramework` follows the same constructor rule: `new(...)`, the startup-report entry, and all plugin-extension entries require the caller's `TaskPool`. They never call `TaskPool::new`, `TaskPool::try_new`, `TaskPools::process_default`, or `EngineTaskGraph::try_new`. The framework stores a weak execution handle, while the surrounding CoreRuntime remains the lifetime owner and must outlive render work.

`DefaultNavigationManager` follows the same owner rule. Its constructor requires the bake `TaskPool`, the Navigation module declares a `TasksModule` dependency, and its service factory injects the activating Core's TaskGraph worker. Production Navigation source cannot create a `TaskPool`, resolve a process default, or create a nested TaskGraph. Unit tests use a retained, explicitly sized TaskGraph fixture; dropping the fixture drops the manager before its execution owner.

`PlatformDriver` also follows the owner rule across both composition paths. `PlatformModule` and `zircon_app`'s host-provided preference-backend factory upgrade the activation Core and pass `core.task_graph().worker_pool().clone()` into the driver. `with_preference_storage_backend(pool, backend)` is the only preinstalled-backend constructor and cannot choose a process owner internally; there is no `Default` implementation. The crate-private persistence adapter likewise accepts its pool through `with_pool(...)` and exposes no implicit constructor. Platform unit fixtures name and retain a one-worker I/O pool, with the driver, manager, or adapter declared first so it is dropped before that owner. This hard cut changes ownership, not preference ordering, window state, host lifecycle, or event-loop scheduling semantics.

`bake_font_sdf_artifact(generation_pool, font_bytes, request)` follows the standalone-tool form of the same rule. It borrows the execution owner for one synchronous batch and reports that pool's parallelism; it cannot create or resolve another worker set. The CLI creates one default-sized `EngineTaskGraph` only after argument parsing and font I/O, passes its worker pool to the bake, and must complete `shutdown(...)` before publishing the artifact. Offline integration tests use a retained two-worker graph. This is an execution-topology hard cut, not a change to glyph selection, SDF generation, packing, encoding, or deterministic artifact identity.

`JobHandle::wait()` is deadlock-resistant when invoked from a runtime worker. The handle wait loop drops its state lock, calls the task-pool-owned `assist_current_thread_once(...)`, and only parks briefly when the current Rayon worker finds no ready task. This keeps direct Rayon calls in the existing `pool.rs` owner and prevents a single-worker scheduler from blocking forever while the only worker waits on a child job it just queued.

`TaskGraphScope::submit(...)`, `schedule(...)`, and `schedule_after(...)` all
return `TaskHandle`. Releasing its final client handle requests cooperative
cancellation only for `CancelOnDrop`; clones retain that client lease, and
`DetachOnDrop` / `FinishOnShutdown` do not request it. A running closure queries
`TaskCancellationToken::is_cancellation_requested()` and must call
`acknowledge_cancellation()` before returning if it actually stops for that
request. A request by itself is not a cancellation receipt: normally returned
work that ignores it is completed. The scope keeps the record in its census
until the task body reaches a terminal state, so client-handle drop cannot evade
scope accounting. `wait_until_quiescent(...)` drains one closed scope without
stopping the shared worker set.

`EngineTaskGraph::shutdown(...)` closes scope admission, waits for admitted task bodies, closes its sole Runtime-owned worker set, releases the sole Rayon owner, and joins every custom-spawned worker handle. Cloned pool/scheduler handles keep only a `Weak` backend route. Each call checks the shared atomic admission gate before and after upgrading that route, so an admitted `install`, `join`, or enqueue temporarily retains the backend without a hot-path mutex, while a post-close call is rejected. Spawned unscoped work remains visible indirectly through worker join: Rayon delays termination until that work returns, and a deadline expiry yields `TaskGraphShutdownError` with an incomplete worker census instead of a false stopped state. Retained handles cannot reopen admission; a later shutdown call continues the same `Closing` transition. Only a report with quiescent scopes and `worker_shutdown.all_joined()` is an unload-capable receipt for this worker set.

Handle-backed scheduled tasks are panic-safe at the synchronization boundary.
If admitted TaskGraph work panics, its `TaskStatus` becomes `Failed`, its
`TaskHandle` reaches completion, and `wait()` reports the panic on the caller
thread. Scoped `schedule_after` propagates prerequisite failure/cancellation
without running the dependent body. Lower-level `JobScheduler::schedule_after`
and `JobHandle::combine` retain the same fence behavior for unscoped work.

`JobHandle::on_terminal(...)` registers a general one-shot observer for successful, panicked, or dependency-cancelled terminal state. Registration admits the observer to asynchronous task-pool delivery; neither a terminal producer nor a late registration executes arbitrary observer code inline. Terminal publication admits existing dependency continuations in their original order, then admits queued observers after those continuations have run. The dispatcher rotates callback envelopes with a bounded per-envelope quantum and a bounded run budget, using at most two existing pool workers; a single origin envelope stays serial. Standalone handles and timers share one process dispatcher state, while every scheduler retains the dispatcher bound to its explicitly supplied pool. Thus a deep dependency failure or a wide observer fan-out cannot consume the producer's native stack or completion call while unrelated envelopes can continue on available pool parallelism. `wait()` synchronizes terminal state rather than observer completion, so observers must own any stronger consumer completion signal. Observer and continuation panics are contained, and an observer panic increments only `JobHandle::terminal_observer_panic_count()` without rewriting the task panic or dependency result.

`TaskTimer` owns deadline selection only. At expiry it admits the registration to the same bounded callback-delivery contract and rechecks cancellation and timer liveness immediately before invocation; the timer control thread never runs a consumer callback body. A periodic registration has at most one admitted or running delivery, so ticks that arrive while its preceding callback is pending are coalesced instead of building an unbounded queue.

The observer is deliberately application-neutral. Runtime 11 does not import winit, the dynamic API, or host cadence policy, and it does not wake the scheduler or event loop for every completed job. A subsystem that owns frame-visible asynchronous output may attach an observer and route its own session-scoped wake; invisible jobs attach none.

`parallel_for` is blocking and uses an explicit chunk size. A chunk size of zero is normalized to one item per chunk. `TaskPool` also implements the framework-neutral `ParallelSliceExecutor` contract through the same implementation, allowing framework algorithms to request slice parallelism without depending on the runtime task module. Callers use it when they need stable completion before continuing the current frame; longer lived work should be scheduled with handles instead.

`ScheduleParallelExecutor` is the first runtime consumer of dependency scheduling. It chains every `ScheduleParallelBatch` from the previous batch handle, records the report counts up front, waits on the final batch handle, and then replays each batch result in source order to keep deterministic error reporting.

## Observability

`JobScheduler` diagnostics are off by default; callers that need lifecycle telemetry construct the scheduler with `with_diagnostics()` before submitting work. The enabled scheduler clones share a fixed 64-shard diagnostics state. A submitting or worker thread receives one cache-aligned shard, so lifecycle updates do not contend on a scheduler-global writer counter. Each shard retains the acquire/release retirement chain and a bounded 16-attempt stable read. Frame reporting then verifies the epoch and retirement state of the full shard set after merging, with a separately bounded aggregate retry; it publishes one complete aggregate snapshot under a single short cache lock, or returns the preceding complete aggregate while writers continue to mutate. Work admitted while diagnostics are off remains untracked even if collection is enabled later, so a terminal event can never appear without its matching admission. `tasks.dependency_waiting`, `tasks.queued`, `tasks.active`, and `tasks.completed` are derived from the merged counters and conserve `tasks.scheduled`; cumulative `tasks.queue_wait_ms` is paired with the same stable started count exposed as `tasks.queue_wait_samples`. Cancellation before worker start has no execution sample. Confirmed cancellation after start records execution time and a private `cancelled_after_start` retirement count so `tasks.active` reaches zero without confusing the two cancellation phases. Detached work uses an unwind-safe completion guard, so its terminal event remains accurate while Rayon retains ownership of panic handling.

`tasks.dependency_wait_ms` remains the separate submission-to-dependency-release duration for `schedule_after`. `JobHandle::wait()` and `JobScheduler::wait_all(...)` now record `tasks.explicit_wait_ms`. The previous `tasks.main_thread_wait_ms` name was removed rather than aliased because the handle can be waited from any thread and the scheduler has no authoritative caller-thread identity; consumers must not infer a main-thread stall from an explicit synchronization duration.

`JobScheduler::diagnostic_report()` exposes an in-memory `JobSchedulerReport`; `JobScheduler::record_diagnostics(store, frame)` publishes the same values into `DiagnosticStore` with `tasks` and `job_scheduler` tags.

`JobScheduler::task_diagnostic_source()` enables an observation-only flag on the shared diagnostics state and returns a runtime-neutral terminal observation stream; it does not activate full lifecycle counter/timing sampling. The stream retains at most 256 panic/cancellation records, bounds messages to 4 KiB at UTF-8 boundaries, and serves at most 64 records per cursor read. Scheduler/task identity is distinct from the monotonic observation cursor, so consumers can deduplicate without turning the runtime into another log owner. Success does not enter this stream; only panic/cancellation takes the bounded journal lock. The editor host owns its cursor and maps new records into the canonical `EditorLogService` as `LogSource::runtime()`; lag is surfaced as one gap warning, not an unbounded replay queue or a second retained log store.

Asset request accounting remains in the asset diagnostic namespace because the orchestration layer still owns admission, de-duplication, completion fanout, and frame deltas. `asset.worker.budgeted_threads` mirrors the shared TaskGraph worker parallelism for correlation; it is not another allocation and must not be added to the `EngineTaskGraphOptions` budget.

## Test Coverage

`zircon_runtime/src/tests/tasks.rs` and the private task modules own the M1/M3 behavior anchors. The current inventory keeps `behavior_test_anchor_count = 73`; the six canonical-handle additions cover single-handle binding, the concurrent terminal/fence invariant, fence/status agreement, prerequisite lease retention, and post-join late observation. The established terminal-observation, bounded-stream, retained-result, scope, dependency, queue-pressure, conserved-snapshot, explicit-wait, cancellation, barrier, and parallel tests continue to prevent structural drift while Cargo validation remains pending. `callback_dispatcher.rs`, `job_handle.rs`, `job_scheduler/tests.rs`, and `timer.rs` additionally own bounded delivery and cancellation regressions. `asset/tests/pipeline/worker_pool.rs` owns the M2.4 budget-accounting anchors:

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
- `cancelled_handle_reports_typed_terminal_state_without_panicking_wait`
- `schedule_after_propagates_dependency_cancellation_without_running_dependent_work`
- `running_work_that_ignores_a_cancellation_request_completes`
- `running_work_reports_cancelled_only_after_acknowledgement`
- `scoped_dynamic_scene_prepare_cancels_before_a_queued_loader_starts`

Cargo execution reached package compilation but did not reach the task tests on 2026-06-13: `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture` first hit a plugin native-loader test import error for `PluginInterfaceManifest`. The missing import has been fixed. A 2026-06-20 clean-window rerun of `cargo test -p zircon_runtime --lib tasks --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-11-validation-0620 --message-format short --color never -- --test-threads=1 --nocapture` stayed in `zircon_runtime` lib-test compilation for the 1200s tool window plus an additional 650s wait and produced no test binary or test result; the residual Cargo/rustc processes from that run were stopped. A narrower 2026-06-20 core-min rerun, `cargo test -p zircon_runtime --lib tasks --no-default-features --features core-min --locked --jobs 1 --target-dir E:\Git\ZirconEngine\target\codex-runtime11-coremin-0620 --message-format short --color never -- --test-threads=1 --nocapture`, also timed out after 1200s during `zircon_runtime` lib-test compilation, produced no `zircon_runtime*.exe` test binary in that target directory, and had matching residual Cargo/rustc command lines stopped. The required milestone commands remain recorded in Runtime 11, and these timeout records do not count as Cargo passes.

The 2026-06-20 lightweight guard pass confirms the static boundary while Cargo remains pending: standalone `job_system.rs` passed 1/1, standalone `rayon_boundary.rs` passed 3/3, standalone `asset_worker_policy.rs` passed 1/1, and `asset_worker_policy.rs` passed rustfmt. The asset worker guard was tightened to inspect the `impl AssetWorkerPool` block for the retired `AssetWorkerPool::new(worker_count)` signature while still requiring `AssetWorkerPoolOptions` to own worker-count configuration, so the guard no longer mistakes the valid `AssetWorkerPoolOptions::new(worker_count)` constructor for the retired pool API.

The core-min window added another lightweight evidence pass before status sync: `job_system_boundary.py` compiled, direct `job_system_boundary_audit` reported `expected_module_count = 9`, `direct_rayon_paths = 2`, `behavior_test_anchor_count = 12`, `missing_behavior_test_anchors = []`, and `risks = []`, and standalone `job_system.rs` 1/1 plus standalone `rayon_boundary.rs` 3/3 passed.

The 2026-06-21 inventory split compiled `job_system_boundary.py`, `job_system_source_inventory.py`, and `job_system_anchor_inventory.py`; direct `job_system_boundary_audit` continued to report task owner modules 9/9, direct Rayon paths 2/2, diagnostic anchors 4/4, behavior-test anchors 12/12, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. The follow-up Markdown renderer split also compiled `job_system_markdown.py`, moved `render_job_system_boundary_markdown` out of `job_system_boundary.py`, and left the direct audit counts unchanged at `risks = []`.

The 2026-06-21 worker wait-assist slice adds `worker_thread_wait_does_not_deadlock_scheduler`, bringing Runtime 11 behavior-test anchors to 13/13. `pool.rs` remains one of the two direct-Rayon owners and now exposes `assist_current_thread_once(...)`; `job_handle.rs` uses that helper plus `WORKER_WAIT_IDLE_PARK` to avoid self-deadlock without adding another Rayon owner path. Standalone `job_system.rs` 1/1, standalone `rayon_boundary.rs` 3/3, and standalone `plan_status.rs` 33/33 remain the lightweight guards for this lane until package-level `tasks/ecs_schedule/worker_pool/rayon` Cargo gates can run.

`runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass` keeps the `tasks/ecs_schedule/worker_pool/rayon` validation lane visible across Runtime 11, the runtime index, Runtime 05 closeout, this module doc, and the M0 review. The render-owned `parallel_frustum.rs` direct-Rayon cutover is complete at static/source level, but Runtime 11 remains `in_progress` until the declared package filters have real Cargo evidence.
