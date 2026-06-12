---
related_code:
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/types.rs
implementation_files:
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
tests:
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_completes_builtin_texture_requests
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_unbounded_mode_is_explicit_opt_in
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_bounded_queue_rejects_overflow_with_explicit_error
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::concurrent_requests_for_same_asset_decode_once_and_notify_all
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs::worker_pool_diagnostics_track_in_flight_and_failure_counts
doc_type: module-detail
---

# Asset Worker Pool

`zircon_runtime::asset::pipeline::worker_pool` owns the CPU decode worker pool for asset requests that can run outside the main runtime path. It is a runtime asset execution backend, not a second asset manager. `ProjectAssetManager::spawn_worker_pool()` is the production owner that supplies the default worker count.

## Options

`AssetWorkerPoolOptions` is the construction contract:

- `worker_count` is normalized to at least one worker.
- `queue_depth: None` is the explicit unbounded mode and preserves the original request/completion channel behavior.
- `queue_depth: Some(n)` creates a bounded request channel with depth `n`.

The old `AssetWorkerPool::new(worker_count)` signature is retired. Callers now construct `AssetWorkerPoolOptions` and pass it to `AssetWorkerPool::new(...)`, so future config-store integration has a single options object to extend.

## Backpressure

`AssetWorkerPool::request(...)` uses non-blocking `try_send(...)`. In bounded mode, a full request queue returns `ZirconError::ChannelSend("asset request queue full: ...")` instead of blocking the caller thread. A disconnected request channel still returns the existing dropped-request `ChannelSend` error.

`request_sender()` remains a low-level escape hatch for existing channel consumers. It now returns the same bounded or unbounded sender created from the pool options, so bounded capacity is still enforced by the channel itself. Direct sender access bypasses the pool's in-flight de-duplication counter, so it is documented as a low-level path rather than the recommended loading API. Runtime 04 leaves that method in place for now because there are no production callers outside this pool boundary; removing it can be a later hard cutover once a real manager-level loading queue exists.

## Request De-Duplication

`AssetWorkerPool` tracks in-flight requests by `AssetRequest`. The first request for a key is registered with waiter count `1` before it is published to the worker channel; if bounded enqueue fails, that registration is rolled back. This ordering prevents a fast worker from completing before the in-flight table knows about the request. Additional requests for the same key increment the waiter count and do not enqueue another decode job. When a worker publishes the completion payload, the pool removes the in-flight key and sends the same payload once per waiter count.

This is intentionally pool-local coalescing. It does not persist cache state, does not deduplicate after a completion has already been published, and does not merge different locators that happen to resolve to the same project asset. Those higher-level decisions belong in `ProjectAssetManager` or project registry resolution.

## Diagnostics

`AssetWorkerPoolDiagnostics` exposes four counters:

- `in_flight`: current waiting request count, including coalesced waiters.
- `completed`: number of completion notifications published.
- `failed`: number of failed completion notifications published.
- `queue_peak`: highest observed waiting request count.

`AssetWorkerPool::record_diagnostics(store, frame_index)` records those counters into the existing `DiagnosticStore` paths `asset.worker.in_flight`, `asset.worker.completed`, `asset.worker.failed`, and `asset.worker.queue_peak`, tagged as `asset` and `worker` with unit `request`.

## Decode Loop

Worker threads are named `zircon-asset-{index}`. Each worker receives an `AssetRequest`, calls the current mesh or texture CPU loader, and publishes either a typed `CpuAssetPayload` or `CpuAssetPayload::Failure { request, message }` to the completion receiver. Dropping the pool closes the request sender and joins all worker threads.

## Validation

The focused worker-pool tests cover the original builtin texture completion path, explicit unbounded mode, deterministic bounded-overflow behavior, duplicate request coalescing, queue-full rollback diagnostics, and diagnostics publication through `DiagnosticStore`. The overflow and coalescing tests use a test-only workerless constructor to keep the receiving side alive without starting a worker, making queue state deterministic and avoiding timing races with real worker consumption.
