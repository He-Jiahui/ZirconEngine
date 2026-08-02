---
related_code:
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/load/texture.rs
  - zircon_runtime/src/asset/load/mesh.rs
  - zircon_runtime/src/asset/formats/obj/error.rs
  - zircon_runtime/src/asset/formats/obj/decode_obj_file.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/worker_pool/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
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
  - zircon_runtime/src/asset/pipeline/worker_pool/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
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
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-22-asset-worker-shared-completion-backpressure.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
tests:
  - tools/tests/test_frameworks_02_core_error_single_source.py
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_completes_builtin_texture_requests_on_the_runtime_io_pool
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_default_budgets_are_hard_limits
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::project_asset_manager_uses_the_injected_runtime_io_pool
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::project_asset_manager_defaults_share_the_process_io_pool
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_bounded_queue_rejects_overflow_with_explicit_error
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::concurrent_requests_for_same_asset_share_one_immutable_payload_owner
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_diagnostics_track_in_flight_and_failure_counts
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_diagnostics_record_queue_age_clone_and_cancel_wall
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_frame_sampler_records_per_job_completion_deltas
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::completion_deadline_replaces_the_pending_request_deadline
  - zircon_runtime/src/asset/pipeline/worker_pool/tests.rs::payload_size_matrix_keeps_one_owner_and_rejects_oversize_retention
  - zircon_runtime/src/asset/pipeline/worker_pool/tests.rs::payload_256_mib_matrix_rejects_oversize_retention [ignored pressure matrix]
  - zircon_runtime/src/asset/pipeline/worker_pool/tests.rs::runtime11_pressure_matrix_records_shared_completion_backpressure [ignored pressure matrix]
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::dropping_worker_pool_cancels_pending_jobs_without_synchronous_wait
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::dropping_worker_pool_preserves_cancelled_ticket_after_armed_deadline
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::dropping_worker_pool_on_its_io_worker_cancels_its_queued_ticket
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

Current Runtime 04 owner sync (2026-07-10): `expected_source_file_count = 25`, `expected_guard_file_count = 22`, `test_anchor_count = 28`, `behavior_test_anchor_count = 24`, `missing_behavior_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The current folder-backed worker-policy and Cargo-gate children are counted explicitly; this supersedes the earlier 11-owner historical mirror without changing worker-pool behavior.

`zircon_runtime::asset::pipeline::worker_pool` owns CPU decode request orchestration. It is a runtime asset execution backend, not a second asset manager and not a second thread-pool owner. `ProjectAssetManager::spawn_worker_pool_with_frame_sampler()` supplies the runtime-owned IO `TaskPool` used for every decode job and its frame telemetry cursor.

## Options

`AssetWorkerPoolOptions` is the construction contract:

- `queue_depth: None` selects the same bounded default admission limit as `AssetWorkerPoolOptions::default()`.
- `queue_depth: Some(n)` admits at most IO-pool parallelism plus `n` unique scheduled requests. Duplicate waiters do not consume another unique-request slot, but each consumes the shared `waiter_capacity` budget.
- `completion_entry_capacity`, `completion_byte_capacity`, `request_max_age`, and `completion_max_age` bound retained unharvested terminal state.
- Worker count and thread-budget source are intentionally absent. The injected `TaskPool` is the execution and budget authority.

The current constructor is `AssetWorkerPool::new(task_pool, options)`. It accepts only an IO-kind pool and does not create dedicated threads. The old worker-count constructor, `from_task_pool_options(...)` bridge, explicit budget override, raw request sender, and test-only workerless constructor are retired.

## Thread Budget

Runtime 11 M2.4 now uses a single process-wide task owner. `TaskPools::default()` resolves through a `OnceLock<TaskPools>`; every default `CoreRuntime` and `ProjectAssetManager` clones the same pool handles. Explicit `TaskPoolOptions::create_pools()` remains the isolated-owner entry for callers that deliberately need a separate task resource set.

`ProjectAssetManager::new(io_task_pool)` is the explicit injection entry. `ProjectAssetManager::default()` uses the process-wide task owner and therefore shares the same IO pool as `CoreRuntime::new()`. `AssetWorkerPool` submits each unique decode through `TaskPool::spawn(...)`; it does not create dedicated threads. `AssetWorkerThreadBudgetSource` has only `TaskPoolIo`, and `asset.worker.budgeted_threads` reports the shared IO pool's parallelism rather than a second charged thread count.

`ProjectAssetManager::spawn_worker_pool_with_frame_sampler()` is the manager-owned construction entry for runtime frame owners that need worker-pool telemetry immediately. It creates the `AssetWorkerPool` and returns an `AssetWorkerPoolFrameSampler` initialized from that pool's current cumulative counters. The former pool-only manager construction entry is retired so production creation keeps the sampler contract explicit.

## Backpressure

`AssetWorkerPool::request(...)` performs non-blocking admission and returns `CoreResult<AssetWorkerCompletionTicket>`. In bounded mode, the unique scheduled-request capacity is `io_parallelism + queue_depth`; exceeding it returns `CoreError::ChannelSend("asset request queue full: ...")` instead of blocking the caller thread. A second observer of the same request receives another ticket only while the shared waiter budget permits it. Accepted requests are submitted to the runtime IO pool immediately and may wait in that pool's scheduler behind other IO work.

Each ticket observes one shared `Arc<CpuAssetPayload>` terminal result. Completion retention is independently bounded by entry count, byte count, and age; an over-budget completion terminalizes as rejected instead of blocking an IO worker on a channel send. `TaskTimer` schedules request and completion deadlines through the Runtime11 task owner. Replacing a deadline invalidates the prior timer generation before registering the new deadline, so a ready result remains valid until its completion deadline, not its earlier request deadline.

`AssetWorkerPool::request(...)` is the only public request entry. The former `request_sender()` channel escape hatch is retired, so callers cannot bypass in-flight coalescing, bounded admission, or worker diagnostics.

## Request De-Duplication

`AssetWorkerPool` tracks in-flight requests by `AssetRequest`. Capacity is checked before a new key is registered. The first accepted request for a key is registered with waiter count `1` before its IO task is spawned, preventing a fast task from completing before the in-flight table knows about the request. Additional requests for the same key increment the waiter count and do not spawn another decode job. Completion publishes one immutable `Arc` payload; tickets observe that owner without per-waiter payload copies. Unharvested results remain in the bounded completion registry until a ticket harvests, cancellation removes them, or their completion deadline expires.

This is intentionally pool-local coalescing. It does not persist cache state, does not deduplicate after a completion has already been published, and does not merge different locators that happen to resolve to the same project asset. Those higher-level decisions belong in `ProjectAssetManager` or project registry resolution.

## Diagnostics

`AssetWorkerPoolDiagnostics` exposes the shared execution source plus bounded-lifecycle counters:

- `thread_budget_source`: always `TaskPoolIo`.
- `budgeted_threads`: parallelism of the shared runtime IO pool; it is not an additional thread allocation.
- `in_flight` and `in_flight_waiters`: current unique work and ticket observers.
- `completed`, `failed`, and `merged`: terminal and de-duplication counters.
- `rejected`, `queue_rejected`, `waiter_rejected`, and `completion_rejected`: hard-budget rejection reasons.
- `expired` and `cancelled`: deadline and lifecycle terminalization reasons.
- `completion_entries` and `completion_bytes`: current retained-result budget consumption.
- `queue_peak`: highest observed unique scheduled-request count.
- `queue_age_total`, `queue_age_max`, and `queue_age_samples`: enqueue-to-worker-start latency, measured only when a queued task begins execution.
- `payload_clone_bytes`: bytes copied from the shared completion payload into ticket results. It remains `0`; ticket delivery only increments the result `Arc` reference count.
- `cancel_wall_*` and `drop_wall_*`: cumulative, maximum, and sample-count wall measurements for non-blocking cancellation and pool teardown.

`AssetWorkerPool::record_diagnostics(store, frame_index)` records request counters into the existing `DiagnosticStore` paths `asset.worker.in_flight`, `asset.worker.completed`, `asset.worker.failed`, and `asset.worker.queue_peak`, tagged as `asset` and `worker` with unit `request`. It also records `asset.worker.budgeted_threads` with unit `thread`, queue-age and lifecycle wall paths with unit `millisecond`, and `asset.worker.payload_clone_bytes` with unit `byte`.

`AssetWorkerPoolFrameSampler` is the frame-local sampling cursor for Runtime 07 performance evidence. It reads cumulative `AssetWorkerPoolDiagnostics` without mutating the pool, then emits `AssetWorkerPoolFrameDiagnostics` for the caller's frame. The frame sample keeps current `in_flight` and `budgeted_threads`, and converts cumulative completion counters into `asset.worker.frame_completed` and `asset.worker.frame_failed` deltas. Multiple owners can keep independent sampler cursors around the same pool, so the worker pool does not gain a single global "last frame" state.

Runtime 15 M3 asset worker pool lock poison recovery keeps those counters available after a panic while a worker-pool lock is held. `AssetWorkerPool::request(...)`, `diagnostics()`, `record_in_flight_locked(...)`, and `publish_completion(...)` now consume poison-recovery helpers for the in-flight map and diagnostics state instead of panicking on `lock poisoned`. The `AssetManager` service contract also reuses `ProjectAssetManager` importer and subscriber lock helpers for open/subscribe paths, so it inherits the same poison recovery policy as the manager runtime owner.

## Decode Loop

Each accepted unique request runs on a runtime IO-pool thread, calls the current mesh or texture CPU loader, and publishes either a typed `CpuAssetPayload` or `CpuAssetPayload::Failure { request, message }` into its shared ticket entry. A panic is converted into a failure payload so lifecycle accounting still reaches a terminal state. `cancel(...)`, deadline expiry, and pool Drop terminalize queued, running, and retained-completion tickets without synchronously joining the shared IO pool; a decode that is already executing may finish, but its result is discarded after its ticket has terminalized. Neither path shuts down or joins the shared process task owner.

Runtime 15 F5 texture loader typed errors keeps `asset/load/texture.rs` failures typed as `TextureLoadError::OpenImage` until `process_request(...)` converts them to `CpuAssetPayload::Failure { message }`. The worker pool is the lossy reporting boundary for async completion consumers; the texture loader itself preserves the `image::ImageError` source. `asset/tests/load/texture.rs::missing_image_file_reports_typed_texture_load_error` and `review_f5_texture_loader_uses_typed_error` lock this `asset/load/texture.rs` / `asset/pipeline/worker_pool.rs` split under `runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred`.

Runtime 15 F5 mesh loader typed errors keeps `asset/load/mesh.rs` failures typed as `MeshLoadError::{UnsupportedFormat, Obj}` until `process_request(...)` converts them to `CpuAssetPayload::Failure { message }`. The OBJ decoder owns its own source-preserving `ObjDecodeError` variants in `asset/formats/obj/error.rs`, including `ObjDecodeError::Read` and parse/index/face/empty-mesh variants. `asset/tests/load/mesh.rs::unsupported_mesh_file_reports_typed_mesh_load_error`, `asset/tests/formats/obj.rs::obj_decode_reports_typed_read_error_source`, `asset/tests/formats/obj.rs::obj_decode_reports_typed_scalar_parse_error_source`, and `review_f5_mesh_loader_and_obj_decoder_use_typed_errors` lock this `asset/load/mesh.rs` / `asset/formats/obj/error.rs` / worker boundary split under `runtime_15_mesh_loader_typed_errors_static_passed_cargo_deferred`.

## Validation

The focused worker-pool tests cover builtin texture completion on an injected IO pool, process-owner sharing, paired manager/sampler construction, deterministic bounded overflow, duplicate request coalescing, shared immutable payload ownership, waiter budgets at 1/1k/100k, a 4 KiB shared-owner case, and a small-budget oversize-retention rejection with the same admission branch used for the 256 MiB matrix. They also assert queue age, zero payload-clone bytes, and cancellation/drop wall diagnostics. `payload_256_mib_matrix_rejects_oversize_retention` and `runtime11_pressure_matrix_records_shared_completion_backpressure` are intentionally ignored and must be run with `--ignored` as explicit RSS/backpressure evidence; they are not counted as ordinary focused-test evidence. The pressure matrix prints RSS plus queue age, clone bytes, and cancellation/drop wall values for waiter 1/1k/100k, worker 1/8/64, and stalled-consumer 0/1/60 second samples. Entry/byte/age completion budgets, request-to-completion deadline replacement, and non-blocking Drop cancellation retain regular focused coverage. Queue and lifecycle tests use a real one-thread IO pool, so they validate the production scheduler path without a workerless test bypass.

`asset_worker_pool_accessors_recover_poisoned_locks` deliberately poisons the in-flight and diagnostics mutexes, then verifies request registration, diagnostics readback, and completion publication still recover. `runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool` keeps the worker pool, service contract, Runtime 15 plans, status rows, and this document synchronized under `runtime_15_asset_worker_pool_lock_poison_recovery_static_passed_cargo_deferred`.

The Runtime 04/11 plan-status mirror is protected by `asset_worker_pool_matches_runtime_04_and_11_decisions`, which keeps the worker pool tied to Runtime 04 asset backpressure decisions, the `request(...)` only public request entry, process-wide task ownership, and Runtime 11 IO-pool execution.

The 2026-07-16 managed Windows `worker_pool` result predates the shared-ticket contract and is retained only as historical evidence. The current Runtime11 backpressure repair requires a fresh managed `asset::tests::pipeline::worker_pool` result before its failure handoff can close.

The Runtime 04 structural mirror is split between `asset_pipeline_source_inventory.py`, `asset_pipeline_anchor_inventory.py`, `asset_pipeline_boundary.py`, and `asset_pipeline_markdown.py`. Source/guard file inventory and expected counts live in the source inventory; worker-pool, diagnostic, behavior, doc, and Cargo gate anchors live in the anchor inventory; the boundary remains the audit reader/risk layer at 328 lines, and the Markdown renderer lives in `asset_pipeline_markdown.py` at 117 lines. Current focused evidence reports `expected_source_file_count = 25`, `expected_guard_file_count = 22`, `worker_diagnostic_count = 7`, `expected_worker_diagnostic_count = 7`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 28`, `behavior_test_anchor_count = 24`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `retired_worker_request_sender_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this worker-pool doc, Runtime 04, the runtime index, facade/watcher/artifact/core-resource docs, M0 review, and runtime-interface convergence aligned with those counts. Broader `asset::` and full-lib Cargo gates remain pending.
