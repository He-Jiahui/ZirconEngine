---
title: Editor57 Backend Plan Selection Snapshot Removal
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: release_measurement_pending
---

# Editor57 backend plan selection snapshot removal

## Scope and contract

`RetainedEditorHost::refresh_project_assets` previously called `EditorHostEventController::editor_snapshot()` after accumulating nonempty backend events solely to pass `asset_activity.selected_asset_uuid` to `plan_asset_backend_refresh`. That call builds the scene and both asset surface snapshots, projects asset types, and reads console activity while holding the workbench shell lock. On a 100,000-asset catalog it materializes a full editor snapshot before the plan handles even a single event.

The plan already sets `refresh_selected_asset_details` for every `CatalogChanged` or `ReferenceChanged` event, regardless of UUID. The subsequent selected-UUID comparison set the same flag to `true` again. The planner now takes only active scene identity and backend events, so the host does not build a full editor snapshot for this planning step. All catalog, resource, preview, scene reload, invalidation, and selected-details flags retain their previous values. The existing visual asset path capacity change in `assets/refresh.rs` was preserved.

This slice removes the planning snapshot only. `refresh_selected_asset_details` still builds its own full snapshot in the frozen `assets/refresh/snapshots.rs` path, `sync_asset_details` may trigger presentation reflection, and preview demand may build chrome. It therefore does not establish the native Editor57-G37 navigation, selection, action, or frame p99 gate.

## Regression and performance evidence

- `editor57_backend_plan_is_selection_independent_for_every_editor_change_kind` compares the exact retired planner against the current plan for all five editor change kinds, absent/matching/unrelated change UUIDs, and absent/matching/unrelated selected UUIDs. `editor57_backend_plan_matches_retired_for_mixed_scene_and_resource_changes` also checks scene reload, resource, paint, and presentation flags. Both assert the details-refresh contract for catalog/reference changes.
- Existing host tests cover idle, preview/catalog, and active-scene resource plans after the signature change.
- Ignored managed Windows Release `editor57_hundred_thousand_asset_backend_plan_release_benchmark` creates one real `EditorAssetCatalogGeneration` containing 100,000 visible texture records, installs it in an `EditorHostEventController`, selects the first asset, then compares the retired `editor_snapshot() + selection-aware plan` stage with the current `plan` stage on the same catalog-change event and warm host. Fixture construction and the initial snapshot are outside timing. It uses five alternating warmup pairs and 31 alternating measured pairs, checks paired plan equality, and reports both ordered raw nanosecond sequences and nearest-rank p50/p95/p99 under `PERF_RESULT EDITOR57_100K_BACKEND_PLAN_SELECTION_SNAPSHOT_BENCH_V1`.

The local planning-stage target is current p95 at most 20% of retired p95. This ratio is pending actual managed Release output. With 31 samples, nearest-rank p99 is the maximum observation and is diagnostic. The comparator excludes event draining, cache invalidation, catalog sync, details generation, preview work, retained paint, and native present. No product p95/p99 claim follows from it.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `assets/refresh.rs`, `backend_refresh.rs`, its `optimization_tests.rs`, and three retained asset refresh tests | Rustfmt and scoped diff check passed; independent static review found no blocker; grouped managed Editor compile pending |
| Behavior | `editor57_backend_plan`, existing retained asset refresh plan tests | Grouped managed Editor test pending |
| Performance | ignored `editor57_hundred_thousand_asset_backend_plan_release_benchmark`, 100,000 records, p95 ratio <= 0.20 | Grouped managed Windows Release output pending |
| Product | Editor57-G37 100,000-asset native navigation/selection/action p95/p99 | Pending |
