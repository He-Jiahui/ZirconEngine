---
related_code:
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/debug.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/types.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
implementation_files:
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/debug.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
tests:
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_completes_builtin_texture_requests
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_unbounded_mode_is_explicit_opt_in
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_options_can_derive_threads_from_runtime_io_budget
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::project_asset_manager_default_workers_use_runtime_io_budget_source
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_bounded_queue_rejects_overflow_with_explicit_error
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::concurrent_requests_for_same_asset_decode_once_and_notify_all
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_diagnostics_track_in_flight_and_failure_counts
  - rustfmt --edition 2021 --check zircon_runtime\src\asset\pipeline\worker_pool.rs zircon_runtime\src\graphics\scene\scene_renderer\mesh\mod.rs zircon_runtime\src\graphics\scene\scene_renderer\hzb\hzb_occlusion_culler.rs zircon_runtime\src\ui\component\state_reducer\keyboard.rs zircon_runtime\src\ui\tests\component_catalog\component_state\keyboard.rs (passed after workerless test constructor diagnostics-order repair)
  - cargo test -p zircon_runtime material_keyboard_action_skips_disabled_grouped_selection_options --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-keyboard-routes-0613-coremin --message-format short --color never -- --test-threads=1 --nocapture (passed after workerless test constructor diagnostics-order repair)
doc_type: module-detail
---

# Asset Worker Pool

`zircon_runtime::asset::pipeline::worker_pool` owns the CPU decode worker pool for asset requests that can run outside the main runtime path. It is a runtime asset execution backend, not a second asset manager. `ProjectAssetManager::spawn_worker_pool()` is the production owner that supplies the default worker count.

## Options

`AssetWorkerPoolOptions` is the construction contract:

- `worker_count` is normalized to at least one worker.
- `queue_depth: None` is the explicit unbounded mode and preserves the original request/completion channel behavior.
- `queue_depth: Some(n)` creates a bounded request channel with depth `n`.
- `thread_budget_source` records whether the worker count is explicit or derived from the runtime task-pool IO budget.

The old `AssetWorkerPool::new(worker_count)` signature is retired. Callers now construct `AssetWorkerPoolOptions` and pass it to `AssetWorkerPool::new(...)`, so future config-store integration has a single options object to extend.

## Thread Budget

Runtime 11 M2.4 uses the explicit-accounting route: asset decode workers remain self-managed `zircon-asset-{index}` threads so Runtime 04 backpressure, de-duplication, and completion channel behavior stay unchanged, but the production default worker count is derived from `TaskPoolOptions::default().resolve_thread_counts(...).io_threads`.

`AssetWorkerPoolOptions::from_task_pool_options(...)` is the budget bridge. It produces an options object whose `worker_count` matches the IO lane allocation and whose `thread_budget_source` is `TaskPoolIo`. `ProjectAssetManager::default()` and the asset module factory use that path. `ProjectAssetManager::new(count)` remains an explicit override and keeps `thread_budget_source` as `Explicit`.

## Backpressure

`AssetWorkerPool::request(...)` uses non-blocking `try_send(...)`. In bounded mode, a full request queue returns `ZirconError::ChannelSend("asset request queue full: ...")` instead of blocking the caller thread. A disconnected request channel still returns the existing dropped-request `ChannelSend` error.

`request_sender()` remains a low-level escape hatch for existing channel consumers. It now returns the same bounded or unbounded sender created from the pool options, so bounded capacity is still enforced by the channel itself. Direct sender access bypasses the pool's in-flight de-duplication counter, so it is documented as a low-level path rather than the recommended loading API. Runtime 04 leaves that method in place for now because there are no production callers outside this pool boundary; removing it can be a later hard cutover once a real manager-level loading queue exists.

## Request De-Duplication

`AssetWorkerPool` tracks in-flight requests by `AssetRequest`. The first request for a key is registered with waiter count `1` before it is published to the worker channel; if bounded enqueue fails, that registration is rolled back. This ordering prevents a fast worker from completing before the in-flight table knows about the request. Additional requests for the same key increment the waiter count and do not enqueue another decode job. When a worker publishes the completion payload, the pool removes the in-flight key and sends the same payload once per waiter count.

This is intentionally pool-local coalescing. It does not persist cache state, does not deduplicate after a completion has already been published, and does not merge different locators that happen to resolve to the same project asset. Those higher-level decisions belong in `ProjectAssetManager` or project registry resolution.

## Diagnostics

`AssetWorkerPoolDiagnostics` exposes the worker budget source plus five counters:

- `thread_budget_source`: `Explicit` or `TaskPoolIo`.
- `budgeted_threads`: worker count charged to the asset worker budget.
- `in_flight`: current waiting request count, including coalesced waiters.
- `completed`: number of completion notifications published.
- `failed`: number of failed completion notifications published.
- `queue_peak`: highest observed waiting request count.

`AssetWorkerPool::record_diagnostics(store, frame_index)` records request counters into the existing `DiagnosticStore` paths `asset.worker.in_flight`, `asset.worker.completed`, `asset.worker.failed`, and `asset.worker.queue_peak`, tagged as `asset` and `worker` with unit `request`. It also records `asset.worker.budgeted_threads` with unit `thread` and tags `asset`, `worker`, `budget`, plus the budget-source tag.

## Decode Loop

Worker threads are named `zircon-asset-{index}`. Each worker receives an `AssetRequest`, calls the current mesh or texture CPU loader, and publishes either a typed `CpuAssetPayload` or `CpuAssetPayload::Failure { request, message }` to the completion receiver. Dropping the pool closes the request sender and joins all worker threads.

## Validation

The focused worker-pool tests cover the original builtin texture completion path, explicit unbounded mode, runtime IO-budget derivation, production manager default budget ownership, deterministic bounded-overflow behavior, duplicate request coalescing, queue-full rollback diagnostics, and diagnostics publication through `DiagnosticStore`. The overflow and coalescing tests use a test-only workerless constructor to keep the receiving side alive without starting a worker, making queue state deterministic and avoiding timing races with real worker consumption.

The Runtime 04/11 plan-status mirror is protected by `asset_worker_pool_matches_runtime_04_and_11_decisions`, which keeps the worker pool tied to Runtime 04 asset backpressure decisions and Runtime 11 IO-thread budget accounting.

The test-only `AssetWorkerPool::new_without_workers_for_test(...)` constructor must derive diagnostics from normalized options before moving those options into the returned struct. This keeps the workerless test path aligned with production diagnostics initialization and avoids a move-after-borrow compile error when unrelated lib-test filters compile the asset worker pool.

The Runtime 04 structural mirror is `asset_pipeline_boundary`. Its current focused evidence reports `expected_source_file_count = 19`, `expected_guard_file_count = 11`, `worker_diagnostic_count = 5`, `expected_worker_diagnostic_count = 5`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 22`, `behavior_test_anchor_count = 18`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this worker-pool doc, Runtime 04, the runtime index, facade/watcher/artifact/core-resource docs, M0 review, and runtime-interface convergence aligned with those counts. This is static structure evidence only; broader `asset::` / `worker_pool` Cargo filters remain pending.
