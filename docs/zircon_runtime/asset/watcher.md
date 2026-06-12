---
related_code:
  - zircon_runtime/src/asset/watch/asset_watch_error.rs
  - zircon_runtime/src/asset/watch/asset_watcher.rs
  - zircon_runtime/src/asset/watch/spawn.rs
  - zircon_runtime/src/asset/watch/watch_loop.rs
  - zircon_runtime/src/asset/watch/fold_events.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/tests/watcher.rs
  - zircon_runtime/src/asset/tests/facade/hot_reload.rs
implementation_files:
  - zircon_runtime/src/asset/watch/asset_watch_error.rs
  - zircon_runtime/src/asset/watch/asset_watcher.rs
  - zircon_runtime/src/asset/watch/spawn.rs
  - zircon_runtime/src/asset/watch/watch_loop.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - user: 2026-06-12 runtime architecture implementation
tests:
  - zircon_runtime/src/asset/tests/watcher.rs::rapid_successive_writes_within_debounce_window_emit_single_reload
  - zircon_runtime/src/asset/tests/watcher.rs::watcher_failure_on_removed_directory_surfaces_observable_error
  - zircon_runtime/src/asset/tests/facade/hot_reload.rs::hot_reload_transitions_through_reloading_state_and_emits_modified_event
  - zircon_runtime/src/asset/tests/facade/hot_reload.rs::reload_failure_emits_reload_failed_event_and_lands_failed_state
doc_type: module-detail
---

# Asset Watcher

`zircon_runtime::asset::watch` owns file-system change observation for runtime assets. It does not import assets directly, does not own resource state, and does not synthesize typed asset events. Its job is to turn notify events into folded `AssetChange` values and to report watcher/import observation failures through a separate error surface.

## Debounce

`ASSET_WATCH_DEFAULT_DEBOUNCE` is `120ms`, preserving the existing runtime behavior. `AssetWatcherOptions` carries the debounce value so tests and future config-store wiring can use a shorter deterministic window without changing the production default.

The loop folds all notify events received within one debounce window before calling the asset-change callback. Multiple writes to the same source path in that window collapse into a single `AssetChangeKind::Modified` row for that asset URI. This follows the same owner split as Bevy's file watcher: the watcher is responsible for coalescing file-system noise, while import and resource-state updates stay in the asset manager and resource manager.

## Errors

Watcher errors are represented by `AssetWatchError { assets_root, paths, message }`. Notify `Err(...)` messages are no longer silently discarded by the watch loop. `ProjectAssetManager` also publishes `AssetWatchError` when a watch-triggered project scan or resource sync fails.

`AssetManager::subscribe_asset_watch_errors()` exposes this stream separately from `subscribe_asset_changes()`. Asset changes remain real asset URI changes only; watcher failures are not squeezed into fake `res://` rows.

## Hot Reload Chain

The watcher does not decide whether an asset reload succeeds. Watch changes cause `ProjectAssetManager` to rescan and sync project resources. `core::resource::ResourceManager` owns the state machine:

- `start_reload(...)` moves a ready/error record to `Reloading` and broadcasts `ResourceEventKind::Updated`.
- `register_ready(...)` completes a successful reload and broadcasts `Updated` when the ready record revision changes.
- `fail_reload(...)` moves the record to `Error` and broadcasts `ResourceEventKind::ReloadFailed`.

The typed asset facade maps those resource events to `AssetEvent::Modified` and `AssetEvent::ReloadFailed`. Runtime 04 keeps that projection instead of adding a second watcher-local hot-reload event channel.

## Validation

The watcher tests cover deterministic debounce folding and explicit notify-error publication through the test-only loop harness. The hot-reload facade tests cover `Loaded -> Reloading -> Loaded` with `Modified` events and `Loaded -> Reloading -> Failed` with `ReloadFailed` plus facade failure-state projection.

Cargo verification for Runtime 04 M3 is pending a clean compile window; the first `cargo test -p zircon_runtime --lib watcher --locked ...` attempt on 2026-06-12 timed out during compilation/linking before Rust test results were returned.
