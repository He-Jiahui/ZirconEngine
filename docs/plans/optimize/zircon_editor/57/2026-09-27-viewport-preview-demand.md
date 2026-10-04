---
title: Editor57 Viewport-Bound Asset Preview Demand
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: release_measurement_pending
---

# Editor57 viewport-bound asset preview demand

## Product defect and contract

`RetainedEditorHost::refresh_visible_asset_previews` previously cloned every UUID in each filtered Asset Browser or Activity asset list into a `BTreeSet`, then called `request_preview_refresh` for every one. A 100,000-item folder therefore made 100,000 per-asset preview-admission calls even when the browser painted only a small scroll window. Scrolling to a new window did not itself demand its preview artifacts.

Preview demand now covers the logical rows intersecting the measured content viewport plus two overscan rows on each side, and always includes the selected UUID even when it is offscreen. The selected UUID from each visible surface is requested before viewport rows, so the preview scheduler's admission cap cannot starve an offscreen selection behind lower-sorting visible UUIDs. Duplicate UUIDs across selection and the two surfaces are requested once. Browser list and thumbnail calculations use the same list metrics and thumbnail grid geometry as pointer routing. Activity list/thumbnail demand accounts for folder rows before asset rows. Before pane geometry exists, the first 128 assets and the selection form a bounded initial request; the first content pointer size update and later wheel scroll replace this with the measured window. A wheel event at a clamped edge with unchanged size and scroll offset issues no new preview demand.

Browser pointer movement and native wheel callbacks take the committed `AssetWorkspaceSnapshot` pointer projection, callback pane size, and current content scroll offset. With a measured callback size or cached pane size, the preview-demand stage requests the newly exposed window without building another `EditorChromeSnapshot`. The generic size resolver can still build chrome once if both sizes are missing. Activity's pointer projection omits `visible_folders`, so its callback uses the full chrome snapshot to retain the folder-to-item offset. Backend refresh still uses `build_chrome()` before the bounded UUID selection. This calls `EditorState::snapshot_with_inspector_customizations`, then `AssetWorkspaceState::build_surface_snapshots`; its folder tree build and catalog-folder scan remain proportional to the number of folders. The patch bounds UUID selection and preview-admission calls, not this backend snapshot stage or the full native frame.

## Regression and Release comparison

- `hundred_thousand_asset_preview_demand_tracks_list_and_thumbnail_scroll_windows` builds 100,000 immutable asset rows, passes them through the production Browser `pointer_projection`, then checks initial, middle, and clamped tail list/thumbnail windows; the actual midpoint asset, tail neighbor, offscreen selected item at the head of the request sequence, and a demand count of at most 128.
- `activity_preview_demand_accounts_for_folder_rows_and_unknown_geometry` checks that many Activity folders keep assets outside the first viewport and that unknown geometry requests at most the 128-item prefix plus selection.
- `selected_previews_from_both_surfaces_precede_bounded_rows_without_duplicates` checks that two distinct offscreen selections lead the merged request order, with no duplicate admission calls.
- Ignored Windows Release `editor57_hundred_thousand_asset_preview_demand_release_benchmark` compares the retired all-filtered UUID materialization with the new viewport candidate calculation on the same prepared 100,000-item pointer projection, in list and thumbnail modes at a fixed 900 x 620 midpoint viewport. It performs five warmup pairs and 31 measured pairs, alternating which implementation runs first in successive pairs. It records both raw nanosecond sample sequences, nearest-rank p50/p95/p99, OS, architecture, and package version under `PERF_RESULT EDITOR57_100K_PREVIEW_DEMAND_BENCH_V1`. The pending acceptance target is viewport p95 at most 25% of all-filtered p95, with at most 128 requested UUIDs versus 100,000. The 31-sample p99 is the maximum observation and is diagnostic only.

The comparison times UUID demand construction only. It excludes `build_chrome()`, asset-manager lookup, per-UUID scheduler locking/job submission, retained paint, native scroll dispatch, and present. The Browser preview-demand callback avoids an additional `build_chrome()` when geometry is already known; a full 100,000-asset native scroll/navigation/selection/action p95/p99 baseline for ED57-G37 remains pending. The existing stable asset-workspace snapshot profile measures at most 10,000 items in a separate dirty owner path, so no 100,000-item chrome-build result is claimed here.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs`; `zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/motion.rs`; `zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots/preview_demand_tests.rs` | Rustfmt `--check`, scoped `git diff --check`, UTF-8/LF, final newline, and trailing-whitespace checks passed; Rust test compilation pending combined managed validation |
| Behavior | `hundred_thousand_asset_preview_demand_tracks_list_and_thumbnail_scroll_windows`; `activity_preview_demand_accounts_for_folder_rows_and_unknown_geometry`; `selected_previews_from_both_surfaces_precede_bounded_rows_without_duplicates` | Combined managed Editor test pending |
| Release | ignored `editor57_hundred_thousand_asset_preview_demand_release_benchmark`, marker `EDITOR57_100K_PREVIEW_DEMAND_BENCH_V1`; five alternating warmup pairs and 31 alternating measured pairs | Combined managed Windows Release measurement pending |
| Product | ED57-G37 100,000-item native navigation/selection/action p95/p99 | Pending; this candidate-stage benchmark does not establish it |
