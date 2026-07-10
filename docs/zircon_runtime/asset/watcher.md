---
related_code:
  - zircon_runtime/src/asset/watch/asset_watch_error.rs
  - zircon_runtime/src/asset/watch/asset_watcher.rs
  - zircon_runtime/src/asset/watch/asset_change_construction.rs
  - zircon_runtime/src/asset/watch/spawn.rs
  - zircon_runtime/src/asset/watch/shutdown_on_drop.rs
  - zircon_runtime/src/asset/watch/watch_loop.rs
  - zircon_runtime/src/asset/watch/fold_events.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/tests/watcher.rs
  - zircon_runtime/src/asset/tests/facade/hot_reload.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
implementation_files:
  - zircon_runtime/src/asset/watch/asset_watch_error.rs
  - zircon_runtime/src/asset/watch/asset_watcher.rs
  - zircon_runtime/src/asset/watch/asset_change_construction.rs
  - zircon_runtime/src/asset/watch/spawn.rs
  - zircon_runtime/src/asset/watch/shutdown_on_drop.rs
  - zircon_runtime/src/asset/watch/watch_loop.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - user: 2026-06-12 runtime architecture implementation
tests:
  - zircon_runtime/src/asset/tests/watcher.rs::rapid_successive_writes_within_debounce_window_emit_single_reload
  - zircon_runtime/src/asset/tests/watcher.rs::watcher_failure_on_removed_directory_surfaces_observable_error
  - zircon_runtime/src/asset/tests/facade/hot_reload.rs::hot_reload_transitions_through_reloading_state_and_emits_modified_event
  - zircon_runtime/src/asset/tests/facade/hot_reload.rs::reload_failure_emits_reload_failed_event_and_lands_failed_state
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs::runtime_15_asset_change_construction_uses_owner_name
doc_type: module-detail
---

# Asset Watcher

Current Runtime 04 owner sync (2026-07-10): `expected_source_file_count = 22`, `expected_guard_file_count = 17`, `test_anchor_count = 24`, `behavior_test_anchor_count = 20`, `missing_behavior_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The current child-owner inventory supersedes the earlier 11-owner historical mirror; watcher behavior and its pending Cargo gate are unchanged.

`zircon_runtime::asset::watch` owns file-system change observation for runtime assets. It does not import assets directly, does not own resource state, and does not synthesize typed asset events. Its job is to turn notify events into folded `AssetChange` values and to report watcher/import observation failures through a separate error surface.

## Debounce

`ASSET_WATCH_DEFAULT_DEBOUNCE` is `120ms`, preserving the existing runtime behavior. `AssetWatcherOptions` carries the debounce value so tests and future config-store wiring can use a shorter deterministic window without changing the production default.

The loop folds all notify events received within one debounce window before calling the asset-change callback. Multiple writes to the same source path in that window collapse into a single `AssetChangeKind::Modified` row for that asset URI. This follows the same owner split as Bevy's file watcher: the watcher is responsible for coalescing file-system noise, while import and resource-state updates stay in the asset manager and resource manager.

`AssetChange::new(...)` construction is owned by `asset/watch/asset_change_construction.rs`. The old `asset_change_new.rs` path has been removed so constructor ownership is named by responsibility, not by a migration-scented `_new` suffix.

## Errors

Watcher errors are represented by `AssetWatchError { assets_root, paths, message }`. Notify `Err(...)` messages are no longer silently discarded by the watch loop. `ProjectAssetManager` also publishes `AssetWatchError` when a watch-triggered project scan or resource sync fails.

`AssetManager::subscribe_asset_watch_errors()` exposes this stream separately from `subscribe_asset_changes()`. Asset changes remain real asset URI changes only; watcher failures are not squeezed into fake `res://` rows.

## Shutdown

`AssetWatcher` shutdown is owned by `asset/watch/shutdown_on_drop.rs`. The drop path sends the stop signal and joins the watcher thread so the watch loop exits without exposing an additional public shutdown API.

## Hot Reload Chain

The watcher does not decide whether an asset reload succeeds. Watch changes cause `ProjectAssetManager` to rescan and sync project resources. `core::resource::ResourceManager` owns the state machine:

- `start_reload(...)` moves a ready/error record to `Reloading` and broadcasts `ResourceEventKind::Updated`.
- `register_ready(...)` completes a successful reload and broadcasts `Updated` when the ready record revision changes.
- `fail_reload(...)` moves the record to `Error` and broadcasts `ResourceEventKind::ReloadFailed`.

The typed asset facade maps those resource events to `AssetEvent::Modified` and `AssetEvent::ReloadFailed`. Runtime 04 keeps that projection instead of adding a second watcher-local hot-reload event channel.

## Validation

The watcher tests cover deterministic debounce folding and explicit notify-error publication through the test-only loop harness. The hot-reload facade tests cover `Loaded -> Reloading -> Loaded` with `Modified` events and `Loaded -> Reloading -> Failed` with `ReloadFailed` plus facade failure-state projection.

Cargo verification for Runtime 04 M3 is pending a clean compile window; the first `cargo test -p zircon_runtime --lib watcher --locked ...` attempt on 2026-06-12 timed out during compilation/linking before Rust test results were returned.

The Runtime 04 `asset_pipeline_source_inventory.py` and `asset_pipeline_anchor_inventory.py` now split watcher source/count ownership from watcher anchor ownership, while `asset_pipeline_boundary.py` stays the 328-line audit reader/risk layer and `asset_pipeline_markdown.py` owns Markdown rendering at 117 lines. Current mirror evidence reports `expected_source_file_count = 22`, `expected_guard_file_count = 17`, `worker_diagnostic_count = 7`, `expected_worker_diagnostic_count = 7`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 24`, `behavior_test_anchor_count = 20`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `retired_worker_request_sender_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this watcher doc aligned with Runtime 04, the runtime index, facade/worker/artifact/core-resource docs, M0 review, and runtime-interface convergence; broader `asset::` / `worker_pool` Cargo filters remain pending.

Fresh 2026-06-25 Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover evidence (`Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover` / `runtime_15_asset_watcher_shutdown_on_drop_naming_hard_cutover_static_passed_cargo_deferred`): `asset/watch/drop_impl.rs` has been removed and the drop-time stop signal plus watcher-thread join owner now lives at `asset/watch/shutdown_on_drop.rs`. `asset/watch/mod.rs` mounts only `mod shutdown_on_drop;`. `naming_boundary/runtime_15_m2/asset_dynamic.rs::runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name` pins the missing old file, the new owner/module entry shape, and the Runtime 15/status/docs anchors. Static validation covered rustfmt, old-path/source scans, docs/status anchor scan, whitespace scan, and scoped diff hygiene. Cargo is deferred while active shared cargo/rustc lanes are running and is not claimed as passing.

Fresh 2026-06-25 Runtime 15 M2 asset change construction module naming hard cutover evidence (`Runtime 15 M2 asset change construction module naming hard cutover` / `runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred`): `asset/watch/asset_change_new.rs` has been removed and `AssetChange::new(...)` now lives at `asset/watch/asset_change_construction.rs`. `asset/watch/mod.rs` mounts only `mod asset_change_construction;`, and `fold_events.rs` continues to call `AssetChange::new(...)` without behavior changes. `naming_boundary/runtime_15_m2/asset_dynamic.rs::runtime_15_asset_change_construction_uses_owner_name` pins the missing old file, the new owner/module entry/caller shape, and the Runtime 15/status/docs anchors. Static validation covered rustfmt, old-path/source scans, docs/status anchor scan, whitespace scan, and scoped diff hygiene. Cargo is deferred while active shared cargo/rustc lanes are running and is not claimed as passing.
