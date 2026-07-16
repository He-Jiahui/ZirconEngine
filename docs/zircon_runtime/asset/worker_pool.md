---
related_code:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/load/texture.rs
  - zircon_runtime/src/asset/load/mesh.rs
  - zircon_runtime/src/asset/formats/obj/error.rs
  - zircon_runtime/src/asset/formats/obj/decode_obj_file.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/debug.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/load/texture.rs
  - zircon_runtime/src/asset/tests/load/mesh.rs
  - zircon_runtime/src/asset/tests/formats/obj.rs
  - zircon_runtime/src/asset/pipeline/types.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
implementation_files:
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/load/texture.rs
  - zircon_runtime/src/asset/load/mesh.rs
  - zircon_runtime/src/asset/formats/obj/error.rs
  - zircon_runtime/src/asset/formats/obj/decode_obj_file.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/debug.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/load/texture.rs
  - zircon_runtime/src/asset/tests/load/mesh.rs
  - zircon_runtime/src/asset/tests/formats/obj.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
tests:
  - tools/tests/test_frameworks_02_core_error_single_source.py
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_completes_builtin_texture_requests_on_the_runtime_io_pool
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_unbounded_mode_is_explicit_opt_in
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::project_asset_manager_uses_the_injected_runtime_io_pool
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::project_asset_manager_defaults_share_the_process_io_pool
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_bounded_queue_rejects_overflow_with_explicit_error
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::concurrent_requests_for_same_asset_decode_once_and_notify_all
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_diagnostics_track_in_flight_and_failure_counts
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_frame_sampler_records_per_frame_completion_deltas
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::dropping_worker_pool_waits_for_its_runtime_io_jobs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::dropping_worker_pool_on_its_io_worker_does_not_deadlock_pending_jobs
  - zircon_runtime/src/tests/tasks.rs::isolated_runtime_fixtures_share_the_process_task_owner
  - zircon_runtime/src/tests/tasks.rs::explicit_task_pool_options_create_an_isolated_task_owner
  - zircon_runtime/src/asset/pipeline/worker_pool.rs::tests::asset_worker_pool_accessors_recover_poisoned_locks
  - zircon_runtime/src/asset/tests/load/texture.rs::missing_image_file_reports_typed_texture_load_error
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs::review_f5_texture_loader_uses_typed_error
  - zircon_runtime/src/asset/tests/load/mesh.rs::unsupported_mesh_file_reports_typed_mesh_load_error
  - zircon_runtime/src/asset/tests/formats/obj.rs::obj_decode_reports_typed_read_error_source
  - zircon_runtime/src/asset/tests/formats/obj.rs::obj_decode_reports_typed_scalar_parse_error_source
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs::review_f5_mesh_loader_and_obj_decoder_use_typed_errors
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool
  - cargo test -p zircon_runtime --lib worker_pool --locked -- --test-threads=1 --nocapture (milestone testing stage)
  - cargo test -p zircon_runtime --lib tasks --locked -- --test-threads=1 --nocapture (milestone testing stage)
doc_type: module-detail
---

# Asset Worker Pool

Current Runtime 04 owner sync (2026-07-10): `expected_source_file_count = 22`, `expected_guard_file_count = 17`, `test_anchor_count = 24`, `behavior_test_anchor_count = 20`, `missing_behavior_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The current folder-backed worker-policy and Cargo-gate children are counted explicitly; this supersedes the earlier 11-owner historical mirror without changing worker-pool behavior.

`zircon_runtime::asset::pipeline::worker_pool` owns CPU decode request orchestration. It is a runtime asset execution backend, not a second asset manager and not a second thread-pool owner. `ProjectAssetManager::spawn_worker_pool()` supplies the runtime-owned IO `TaskPool` used for every decode job.

## Options

`AssetWorkerPoolOptions` is the construction contract:

- `queue_depth: None` is the explicit unbounded admission mode.
- `queue_depth: Some(n)` admits at most IO-pool parallelism plus `n` unique in-flight requests. Duplicate waiters for an already admitted request do not consume another queue slot.
- Worker count and thread-budget source are intentionally absent. The injected `TaskPool` is the execution and budget authority.

The current constructor is `AssetWorkerPool::new(task_pool, options)`. It accepts only an IO-kind pool and does not create dedicated threads. The old worker-count constructor, `from_task_pool_options(...)` bridge, explicit budget override, raw request sender, and test-only workerless constructor are retired.

## Thread Budget

Runtime 11 M2.4 now uses a single process-wide task owner. `TaskPools::default()` resolves through a `OnceLock<TaskPools>`; every default `CoreRuntime` and `ProjectAssetManager` clones the same pool handles. Explicit `TaskPoolOptions::create_pools()` remains the isolated-owner entry for callers that deliberately need a separate task resource set.

`ProjectAssetManager::new(io_task_pool)` is the explicit injection entry. `ProjectAssetManager::default()` uses the process-wide task owner and therefore shares the same IO pool as `CoreRuntime::new()`. `AssetWorkerPool` submits each unique decode through `TaskPool::spawn(...)`; it does not create dedicated threads. `AssetWorkerThreadBudgetSource` has only `TaskPoolIo`, and `asset.worker.budgeted_threads` reports the shared IO pool's parallelism rather than a second charged thread count.

`ProjectAssetManager::spawn_worker_pool_with_frame_sampler()` is the manager-owned construction entry for runtime frame owners that need worker-pool telemetry immediately. It creates the `AssetWorkerPool` from the same default options as `spawn_worker_pool()`, then returns an `AssetWorkerPoolFrameSampler` initialized from that pool's current cumulative counters. `spawn_worker_pool()` delegates to the paired constructor and returns only the pool, so production worker creation keeps one options path.

## Backpressure

`AssetWorkerPool::request(...)` performs non-blocking admission and returns `CoreResult<()>`. In bounded mode, the capacity is `io_parallelism + queue_depth`; exceeding that capacity returns `CoreError::ChannelSend("asset request queue full: ...")` instead of blocking the caller thread. Accepted requests are submitted to the runtime IO pool immediately and may wait in that pool's scheduler behind other IO work. The worker does not retain a second asset-specific or compatibility error surface.

`AssetWorkerPool::request(...)` is the only public request entry. The former `request_sender()` channel escape hatch is retired, so callers cannot bypass in-flight coalescing, bounded admission, or worker diagnostics.

## Request De-Duplication

`AssetWorkerPool` tracks in-flight requests by `AssetRequest`. Capacity is checked before a new key is registered. The first accepted request for a key is registered with waiter count `1` before its IO task is spawned, preventing a fast task from completing before the in-flight table knows about the request. Additional requests for the same key increment the waiter count and do not spawn another decode job. Completion removes the key and sends the same payload once per waiter count.

This is intentionally pool-local coalescing. It does not persist cache state, does not deduplicate after a completion has already been published, and does not merge different locators that happen to resolve to the same project asset. Those higher-level decisions belong in `ProjectAssetManager` or project registry resolution.

## Diagnostics

`AssetWorkerPoolDiagnostics` exposes the shared execution source plus five counters:

- `thread_budget_source`: always `TaskPoolIo`.
- `budgeted_threads`: parallelism of the shared runtime IO pool; it is not an additional thread allocation.
- `in_flight`: current waiting request count, including coalesced waiters.
- `completed`: number of completion notifications published.
- `failed`: number of failed completion notifications published.
- `queue_peak`: highest observed waiting request count.

`AssetWorkerPool::record_diagnostics(store, frame_index)` records request counters into the existing `DiagnosticStore` paths `asset.worker.in_flight`, `asset.worker.completed`, `asset.worker.failed`, and `asset.worker.queue_peak`, tagged as `asset` and `worker` with unit `request`. It also records `asset.worker.budgeted_threads` with unit `thread` and tags `asset`, `worker`, `budget`, plus the budget-source tag.

`AssetWorkerPoolFrameSampler` is the frame-local sampling cursor for Runtime 07 performance evidence. It reads cumulative `AssetWorkerPoolDiagnostics` without mutating the pool, then emits `AssetWorkerPoolFrameDiagnostics` for the caller's frame. The frame sample keeps current `in_flight` and `budgeted_threads`, and converts cumulative completion counters into `asset.worker.frame_completed` and `asset.worker.frame_failed` deltas. Multiple owners can keep independent sampler cursors around the same pool, so the worker pool does not gain a single global "last frame" state.

Runtime 15 M3 asset worker pool lock poison recovery keeps those counters available after a panic while a worker-pool lock is held. `AssetWorkerPool::request(...)`, `diagnostics()`, `record_in_flight_locked(...)`, and `publish_completion(...)` now consume poison-recovery helpers for the in-flight map and diagnostics state instead of panicking on `lock poisoned`. The `AssetManager` service contract also reuses `ProjectAssetManager` importer and subscriber lock helpers for open/subscribe paths, so it inherits the same poison recovery policy as the manager runtime owner.

## Decode Loop

Each accepted unique request runs on a runtime IO-pool thread, calls the current mesh or texture CPU loader, and publishes either a typed `CpuAssetPayload` or `CpuAssetPayload::Failure { request, message }` to the completion receiver. A panic is converted into a failure payload so lifecycle accounting still reaches a terminal state. Dropping the orchestration pool off-pool waits on its own pending-job count and condition variable. A drop on the same IO executor returns without blocking its worker so queued jobs can terminalize and release their captured state; neither path shuts down or joins the shared process task owner.

Runtime 15 F5 texture loader typed errors keeps `asset/load/texture.rs` failures typed as `TextureLoadError::OpenImage` until `process_request(...)` converts them to `CpuAssetPayload::Failure { message }`. The worker pool is the lossy reporting boundary for async completion consumers; the texture loader itself preserves the `image::ImageError` source. `asset/tests/load/texture.rs::missing_image_file_reports_typed_texture_load_error` and `review_f5_texture_loader_uses_typed_error` lock this `asset/load/texture.rs` / `asset/pipeline/worker_pool.rs` split under `runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred`.

Runtime 15 F5 mesh loader typed errors keeps `asset/load/mesh.rs` failures typed as `MeshLoadError::{UnsupportedFormat, Obj}` until `process_request(...)` converts them to `CpuAssetPayload::Failure { message }`. The OBJ decoder owns its own source-preserving `ObjDecodeError` variants in `asset/formats/obj/error.rs`, including `ObjDecodeError::Read` and parse/index/face/empty-mesh variants. `asset/tests/load/mesh.rs::unsupported_mesh_file_reports_typed_mesh_load_error`, `asset/tests/formats/obj.rs::obj_decode_reports_typed_read_error_source`, `asset/tests/formats/obj.rs::obj_decode_reports_typed_scalar_parse_error_source`, and `review_f5_mesh_loader_and_obj_decoder_use_typed_errors` lock this `asset/load/mesh.rs` / `asset/formats/obj/error.rs` / worker boundary split under `runtime_15_mesh_loader_typed_errors_static_passed_cargo_deferred`.

## Validation

The focused worker-pool tests cover builtin texture completion on an injected IO pool, process-owner sharing, explicit isolated-pool construction, paired manager/sampler construction, deterministic bounded overflow, duplicate request coalescing, failure diagnostics, frame deltas, off-pool Drop waiting, and same-worker Drop progress. Queue and lifecycle tests use a real one-thread IO pool, so they validate the production scheduler path without a workerless test bypass.

`asset_worker_pool_accessors_recover_poisoned_locks` deliberately poisons the in-flight and diagnostics mutexes, then verifies request registration, diagnostics readback, and completion publication still recover. `runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool` keeps the worker pool, service contract, Runtime 15 plans, status rows, and this document synchronized under `runtime_15_asset_worker_pool_lock_poison_recovery_static_passed_cargo_deferred`.

The Runtime 04/11 plan-status mirror is protected by `asset_worker_pool_matches_runtime_04_and_11_decisions`, which keeps the worker pool tied to Runtime 04 asset backpressure decisions, the `request(...)` only public request entry, process-wide task ownership, and Runtime 11 IO-pool execution.

The 2026-07-16 managed Windows `worker_pool` filter completed with 18 passed, 0 failed, and 8152 filtered in coordinator job `c7c9a84482e34825aa1b0d94a08aee97`. That run includes the real single-worker same-executor Drop regression and the Runtime15 archive-owner guards. A follow-up drop-thread start handshake strengthens the off-pool waiting assertion; its current-source replay stays queued behind an already running foreign managed Editor job and is not used to close the broader Editor full-lib failure.

The Runtime 04 structural mirror is split between `asset_pipeline_source_inventory.py`, `asset_pipeline_anchor_inventory.py`, `asset_pipeline_boundary.py`, and `asset_pipeline_markdown.py`. Source/guard file inventory and expected counts live in the source inventory; worker-pool, diagnostic, behavior, doc, and Cargo gate anchors live in the anchor inventory; the boundary remains the audit reader/risk layer at 328 lines, and the Markdown renderer lives in `asset_pipeline_markdown.py` at 117 lines. Current focused evidence reports `expected_source_file_count = 22`, `expected_guard_file_count = 17`, `worker_diagnostic_count = 7`, `expected_worker_diagnostic_count = 7`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 24`, `behavior_test_anchor_count = 20`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `retired_worker_request_sender_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this worker-pool doc, Runtime 04, the runtime index, facade/watcher/artifact/core-resource docs, M0 review, and runtime-interface convergence aligned with those counts. Broader `asset::` and full-lib Cargo gates remain pending.
