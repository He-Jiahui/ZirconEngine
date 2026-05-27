---
related_code:
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/console_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/hierarchy_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/module_plugins_body.v2.ui.toml
  - zircon_editor/assets/icons/editor_pages/**
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
implementation_files:
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/console_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/hierarchy_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/module_plugins_body.v2.ui.toml
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
  - docs/zircon_editor/assets/editor-page-function-svg-resources.md
  - docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md
  - docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
plan_sources:
  - user: 2026-05-21 continue improving Editor SVGs; choose A + C: polish existing icons, then prepare UI mapping
  - docs/superpowers/specs/2026-05-21-editor-svg-polish-ui-mapping-design.md
  - docs/superpowers/plans/2026-05-21-editor-svg-polish-ui-mapping.md
  - user: 2026-05-23 wire production Editor templates to accepted editor_pages icon mappings
  - docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md
  - docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md
  - user: 2026-05-25 complete live Editor visual rendering and 16px readability validation for wired editor_pages icons
tests:
  - template wiring validation: 39 direct/near rows use expected editor_pages paths, referenced SVGs exist, and 7 gaps remain unchanged
  - asset inventory: 204 SVG files under zircon_editor/assets/icons/editor_pages with page groups A 61, B 54, C 89
  - SVG contract scans: no forbidden constructs and no non-ASCII bytes under zircon_editor/assets/icons/editor_pages
  - repository asset test: cargo test -p zircon_editor --lib repository_assets --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture passed with 1 test, 0 failures, and 1510 filtered on the closeout rerun
  - retained-host 16px readability test: cargo test -p zircon_editor editor_pages_template_icons_have_readable_16px_raster_footprints --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture
  - retained-host screenshot gate: cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --ignored --nocapture
  - live editor build: cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation
  - live editor screenshot: target/visual-layout/editor-live-window-900x620.png
doc_type: module-detail
---

# Editor Page Function Icon Template Map

This document is the production Editor template icon wiring map for the `editor_pages` SVG pack. Accepted `direct` and `near` rows are now wired in production `*.v2.ui.toml` templates using repository-icon-root-relative `editor_pages/...svg` values.

The remaining `gap` rows are intentionally unwired. They keep their existing generic icon names until a future coverage or UI-design pass creates a more specific icon or changes the control role. No `.zui`, icon asset, registry, atlas file, demo template, or showcase template is changed by this wiring map. The later visual validation addendum adds a retained-host raster regression and app/runtime build-boundary fixes so live Editor evidence can be captured; it does not change the 39 wired rows or the 7 deferred gaps.

## Confidence Summary

| Confidence | Count | Wiring status | Meaning |
| --- | ---: | --- | --- |
| `direct` | 22 | wired | Existing `editor_pages` icon matches the current UI role closely enough for production template use. |
| `near` | 17 | wired | Existing icon is usable as a close semantic match, but the role would benefit from a more exact future asset or UI naming review. |
| `gap` | 7 | intentionally unwired | No precise `editor_pages` replacement exists in the current 204-icon pack; defer wiring until a future coverage pass. |

Total mapped production template usages in scope: 46.

## Visual Validation Addendum

The retained-host visual validation closes the original residual risk that static template wiring did not prove compact rendering. `editor_pages_template_icons_have_readable_16px_raster_footprints` renders the unique wired `editor_pages` paths through the production retained-host template icon path at 16 x 16 px, then requires a visible non-full-slot alpha footprint with at least 12 visible pixels and at least a 6 x 6 span. The unique icon set has 29 paths because several template rows intentionally share the same folder, scene, log, lit, open-project, or close-tab icon.

The screenshot gate `capture_m3_gui_acceptance_visual_artifacts` also covers retained-host visual artifacts under `target/visual-layout`, including small and large SVG icon scaling captures. The live Editor smoke build uses `cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation`; the captured native window is `target/visual-layout/editor-live-window-900x620.png`. The file name records the requested capture size, but the actual OS window capture was `1296 x 759` and `86492` bytes.

Two build-boundary fixes were needed before the live Editor binary could compile in this workspace state. `zircon_runtime/src/lib.rs` now exposes the manifest-specific runtime-profile helper functions that `zircon_app` already consumes through the runtime crate root. `zircon_app/src/entry/builtin_modules.rs` clones the optional project manifest before the feature-registration profile fallback so both the plugin-registration and feature-registration resolver paths can use the caller-supplied manifest or profile default without moving the value away from later feature dependency checks.

## Production Template Mapping

| Template path | Control id or local node | Current template icon/value | Current role | Mapped editor_pages asset | Confidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml` | `OpenProject` / `open_project` | `icon: editor_pages/workbench/menu/open-project.svg` | Open a project from the workbench menu bar. | `zircon_editor/assets/icons/editor_pages/workbench/menu/open-project.svg` | `direct` | Wired from former `folder-open-outline`; matches the workbench open-project action. |
| `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml` | `SaveProject` / `save_project` | `icon: editor_pages/workbench/menu/save-all.svg` | Save the current project from the workbench menu bar. | `zircon_editor/assets/icons/editor_pages/workbench/menu/save-all.svg` | `direct` | Wired from former `save-outline`; current route is project-level save, not a single document save. If later narrowed to active-document save, revisit as `near`. |
| `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml` | `ResetLayout` / `reset_layout` | `icon: editor_pages/workbench/dock/reset-layout.svg` | Reset the workbench layout. | `zircon_editor/assets/icons/editor_pages/workbench/dock/reset-layout.svg` | `direct` | Wired from former `grid-outline`; exact dock/layout reset role. |
| `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml` | `AssetsToggle` / `assets_toggle` | `icon: editor_pages/asset_browser/navigation/folder.svg` | Toggle the assets drawer from the activity rail. | `zircon_editor/assets/icons/editor_pages/asset_browser/navigation/folder.svg` | `direct` | Wired from former `albums-outline`; uses the asset browser folder metaphor for the top-level assets surface. |
| `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml` | `HierarchyToggle` / `hierarchy_toggle` | `icon: editor_pages/hierarchy/entity/scene.svg` | Toggle the scene hierarchy drawer. | `zircon_editor/assets/icons/editor_pages/hierarchy/entity/scene.svg` | `direct` | Wired from former `layers-outline`; scene hierarchy is the drawer target. |
| `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml` | `ConsoleToggle` / `console_toggle` | `icon: editor_pages/console_profiler/logs/log-info.svg` | Toggle the console drawer. | `zircon_editor/assets/icons/editor_pages/console_profiler/logs/log-info.svg` | `near` | Wired from former `terminal-outline`; current pack has log/info console coverage but no terminal glyph. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetTool` / `set_tool` | `icon: editor_pages/scene_viewport/tools/universal-transform.svg` | Select or change the active viewport tool. | `zircon_editor/assets/icons/editor_pages/scene_viewport/tools/universal-transform.svg` | `near` | Wired from former `move-outline`; universal transform is close, but this control is a generic tool selector. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetTransformSpace` / `set_transform_space` | `icon: resize-outline` | Change viewport transform space. | `gap` | `gap` | Current pack lacks a local/world transform-space icon. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetDisplayMode` / `set_display_mode` | `icon: editor_pages/scene_viewport/display/lit.svg` | Change viewport display mode. | `zircon_editor/assets/icons/editor_pages/scene_viewport/display/lit.svg` | `near` | Wired from former `eye-outline`; display/lit coverage is close, but a mode-selector or visibility eye variant would be more exact. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetGridMode` / `set_grid_mode` | `icon: editor_pages/scene_viewport/display/grid-overlay.svg` | Change viewport grid mode. | `zircon_editor/assets/icons/editor_pages/scene_viewport/display/grid-overlay.svg` | `direct` | Wired from former `grid-outline`; exact viewport grid display role. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetTranslateSnap` / `set_translate_snap` | `icon: editor_pages/scene_viewport/snapping/grid-snap.svg` | Change translation snapping. | `zircon_editor/assets/icons/editor_pages/scene_viewport/snapping/grid-snap.svg` | `near` | Wired from former `magnet-outline`; existing snap icon covers grid snapping, but not translate-only snapping. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetRotateSnapDegrees` / `set_rotate_snap` | `icon: editor_pages/scene_viewport/snapping/angle-snap.svg` | Change rotation snap degrees. | `zircon_editor/assets/icons/editor_pages/scene_viewport/snapping/angle-snap.svg` | `direct` | Wired from former `sync-outline`; matches angular snapping. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetScaleSnap` / `set_scale_snap` | `icon: editor_pages/scene_viewport/snapping/scale-snap.svg` | Change scale snapping. | `zircon_editor/assets/icons/editor_pages/scene_viewport/snapping/scale-snap.svg` | `direct` | Wired from former `expand-outline`; exact scale snap role. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetPreviewLighting` / `set_preview_lighting` | `icon: editor_pages/scene_viewport/display/lit.svg` | Toggle or change viewport preview lighting. | `zircon_editor/assets/icons/editor_pages/scene_viewport/display/lit.svg` | `near` | Wired from former `sunny-outline`; lit display is close to lighting preview, but not a light-source icon. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetPreviewSkybox` / `set_preview_skybox` | `icon: cloud-outline` | Toggle or change viewport preview skybox. | `gap` | `gap` | Current pack lacks a skybox/cloud/environment icon. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetGizmosEnabled` / `set_gizmos` | `icon: editor_pages/scene_viewport/display/gizmo-visibility.svg` | Toggle viewport gizmo visibility. | `zircon_editor/assets/icons/editor_pages/scene_viewport/display/gizmo-visibility.svg` | `direct` | Wired from former `construct-outline`; exact gizmo visibility role. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `FrameSelection` / `frame_selection` | `icon: editor_pages/scene_viewport/camera/frame-selection.svg` | Frame the current selection in the viewport camera. | `zircon_editor/assets/icons/editor_pages/scene_viewport/camera/frame-selection.svg` | `direct` | Wired from former `scan-outline`; exact camera framing role. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `EnterPlayMode` / `enter_play` | `icon: editor_pages/scene_viewport/play/play.svg` | Enter play mode. | `zircon_editor/assets/icons/editor_pages/scene_viewport/play/play.svg` | `direct` | Wired from former `play-outline`; exact play control role. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `ExitPlayMode` / `exit_play` | `icon: editor_pages/scene_viewport/play/stop.svg` | Exit play mode. | `zircon_editor/assets/icons/editor_pages/scene_viewport/play/stop.svg` | `direct` | Wired from former `stop-outline`; exact stop control role. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetProjectionMode` / `set_projection_mode` | `icon: editor_pages/scene_viewport/camera/perspective.svg` | Change camera projection mode. | `zircon_editor/assets/icons/editor_pages/scene_viewport/camera/perspective.svg` | `near` | Wired from former `camera-outline`; perspective camera is close, but the control is a projection-mode selector. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `AlignView` / `align_view` | `icon: navigate-outline` | Align the viewport camera. | `gap` | `gap` | Current pack has orbit/pan/zoom but no axis/view-align icon. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `SelectFolder` / `select_folder` | `icon: editor_pages/asset_browser/navigation/folder.svg` | Select an asset browser folder. | `zircon_editor/assets/icons/editor_pages/asset_browser/navigation/folder.svg` | `direct` | Wired from former `folder-outline`; exact asset navigation folder role. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `SelectItem` / `select_item` | `icon: cube-outline` | Select a generic asset item. | `gap` | `gap` | Current pack has concrete asset type icons but no generic selected-asset/item icon. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `SetViewMode` / `view_mode` | `icon: list-outline` | Change asset surface view mode. | `gap` | `gap` | Current pack lacks list/grid view-mode icons. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `SetUtilityTab` / `utility_tab` | `icon: editor_pages/asset_browser/import_pipeline/import-settings.svg` | Select asset utility tools. | `zircon_editor/assets/icons/editor_pages/asset_browser/import_pipeline/import-settings.svg` | `near` | Wired from former `options-outline`; import settings is the closest asset-browser tool/settings glyph, but the tab is generic utilities. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `ActivateReference` / `activate_reference` | `icon: editor_pages/asset_browser/references/reference.svg` | Activate or use an asset reference. | `zircon_editor/assets/icons/editor_pages/asset_browser/references/reference.svg` | `direct` | Wired from former `link-outline`; exact reference activation domain. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `OpenAssetBrowser` / `open_browser` | `icon: editor_pages/asset_browser/navigation/folder.svg` | Open the asset browser. | `zircon_editor/assets/icons/editor_pages/asset_browser/navigation/folder.svg` | `near` | Wired from former `albums-outline`; folder identifies the asset browser domain, but it is not an open-browser/library glyph. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `LocateSelectedAsset` / `locate_asset` | `icon: editor_pages/asset_browser/navigation/search.svg` | Locate the selected asset. | `zircon_editor/assets/icons/editor_pages/asset_browser/navigation/search.svg` | `near` | Wired from former `locate-outline`; search is a close navigation action, but a locate/reveal-in-browser icon would be more exact. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `ImportModel` / `import_model` | `icon: editor_pages/asset_browser/import_pipeline/import.svg` | Import a model asset. | `zircon_editor/assets/icons/editor_pages/asset_browser/import_pipeline/import.svg` | `direct` | Wired from former `cloud-upload-outline`; exact import pipeline role despite the prior upload metaphor. |
| `zircon_editor/assets/ui/editor/host/pane_surface_controls.v2.ui.toml` | `TriggerAction` / `trigger_action` | `icon: flash-outline` | Trigger a generic pane action/focus action. | `gap` | `gap` | Current pack lacks a generic flash/action icon, and the route is placeholder-like rather than a stable page/function role. |
| `zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml` | `CreateProject` / `create` | `icon: add-circle-outline` | Create a new project from the welcome surface. | `gap` | `gap` | Current pack lacks project-create/new-project coverage. |
| `zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml` | `OpenExistingProject` / `open_existing` | `icon: editor_pages/workbench/menu/open-project.svg` | Open an existing project from the welcome surface. | `zircon_editor/assets/icons/editor_pages/workbench/menu/open-project.svg` | `direct` | Wired from former `folder-open-outline`; same open-project role as workbench menu. |
| `zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml` | `OpenRecentProject` / `open_recent` | `icon: editor_pages/asset_browser/navigation/recent.svg` | Open a recent project. | `zircon_editor/assets/icons/editor_pages/asset_browser/navigation/recent.svg` | `near` | Wired from former `time-outline`; current pack has a recent-navigation icon, but not project-specific recent coverage. |
| `zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml` | `RemoveRecentProject` / `remove_recent` | `icon: editor_pages/workbench/tabs/close-tab.svg` | Remove a project from the recent list. | `zircon_editor/assets/icons/editor_pages/workbench/tabs/close-tab.svg` | `near` | Wired from former `close-circle-outline`; close-tab conveys removal/close, but a remove-recent or delete icon would be more exact. |
| `zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml` | `ActivityRailButtonIcon0` / `activity_rail_button_icon_0` | `icon: editor_pages/asset_browser/navigation/folder.svg`; `value: editor_pages/asset_browser/navigation/folder.svg` | Static activity rail assets icon. | `zircon_editor/assets/icons/editor_pages/asset_browser/navigation/folder.svg` | `direct` | Wired from former `albums-outline` and `ionicons/albums-outline.svg`; maps the static assets rail icon to the same asset browser folder metaphor. |
| `zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml` | `ActivityRailButtonIcon1` / `activity_rail_button_icon_1` | `icon: editor_pages/hierarchy/entity/scene.svg`; `value: editor_pages/hierarchy/entity/scene.svg` | Static activity rail hierarchy icon. | `zircon_editor/assets/icons/editor_pages/hierarchy/entity/scene.svg` | `direct` | Wired from former `layers-outline` and `ionicons/layers-outline.svg`; static hierarchy rail icon maps to the scene hierarchy icon. |
| `zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml` | `DockTabClose0` / `dock_tab_close_0` | `icon: editor_pages/workbench/tabs/close-tab.svg`; `value: editor_pages/workbench/tabs/close-tab.svg` | Close the first dock tab. | `zircon_editor/assets/icons/editor_pages/workbench/tabs/close-tab.svg` | `direct` | Wired from former `close-outline` and `ionicons/close-outline.svg`; exact tab close role. |
| `zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml` | `DockTabClose1` / `dock_tab_close_1` | `icon: editor_pages/workbench/tabs/close-tab.svg`; `value: editor_pages/workbench/tabs/close-tab.svg` | Close the second dock tab. | `zircon_editor/assets/icons/editor_pages/workbench/tabs/close-tab.svg` | `direct` | Wired from former `close-outline` and `ionicons/close-outline.svg`; exact tab close role. |
| `zircon_editor/assets/ui/editor/host/console_body.v2.ui.toml` | `FocusConsole` / `focus` | `icon: editor_pages/console_profiler/logs/log-info.svg` | Focus the console pane. | `zircon_editor/assets/icons/editor_pages/console_profiler/logs/log-info.svg` | `near` | Wired from former `terminal-outline`; current pack covers logs but not terminal. |
| `zircon_editor/assets/ui/editor/host/hierarchy_body.v2.ui.toml` | `SelectRoot` / `select_root` | `icon: editor_pages/hierarchy/entity/scene.svg` | Select the hierarchy root scene node. | `zircon_editor/assets/icons/editor_pages/hierarchy/entity/scene.svg` | `direct` | Wired from former `layers-outline`; exact scene/root hierarchy role. |
| `zircon_editor/assets/ui/editor/host/animation_graph_body.v2.ui.toml` | `AddNode` / `add_node` | `icon: editor_pages/graph_editor/nodes/state-node.svg` | Add an animation graph node. | `zircon_editor/assets/icons/editor_pages/graph_editor/nodes/state-node.svg` | `near` | Wired from former `plus-circle-outline`; graph node icon covers the domain, but current pack lacks an add-node overlay. |
| `zircon_editor/assets/ui/editor/host/animation_sequence_body.v2.ui.toml` | `ScrubTimeline` / `scrub` | `icon: editor_pages/animation_timeline/transport/timeline-play.svg` | Scrub or play through the animation timeline. | `zircon_editor/assets/icons/editor_pages/animation_timeline/transport/timeline-play.svg` | `near` | Wired from former `play-circle-outline`; timeline play is close, but scrubbing may need a transport/scrubber-specific glyph. |
| `zircon_editor/assets/ui/editor/host/performance_timeline_body.v2.ui.toml` | `RefreshTimelineSnapshot` / `refresh` | `icon: editor_pages/console_profiler/profiling/frame-time.svg` | Refresh or focus the performance timeline snapshot. | `zircon_editor/assets/icons/editor_pages/console_profiler/profiling/frame-time.svg` | `near` | Wired from former `timeline-outline`; frame-time is a profiler view/metre icon, while the current control role is a refresh/focus timeline action. |
| `zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.v2.ui.toml` | `FocusDiagnostics` / `focus` | `icon: editor_pages/console_profiler/diagnostics/watch.svg` | Focus runtime diagnostics. | `zircon_editor/assets/icons/editor_pages/console_profiler/diagnostics/watch.svg` | `near` | Wired from former `pulse-outline`; diagnostics watch is close to pulse/monitoring, but not an exact pulse glyph. |
| `zircon_editor/assets/ui/editor/host/build_export_desktop_body.v2.ui.toml` | `FocusBuildExport` / `focus` | `icon: editor_pages/build_plugins/package/package.svg` | Focus desktop build/export plans. | `zircon_editor/assets/icons/editor_pages/build_plugins/package/package.svg` | `near` | Wired from former `download-outline`; package/export domain is close, but current pack lacks a desktop export/download icon. |
| `zircon_editor/assets/ui/editor/host/module_plugins_body.v2.ui.toml` | `FocusModulePlugins` / `focus` | `icon: editor_pages/build_plugins/plugins/plugin.svg` | Focus project module plugins. | `zircon_editor/assets/icons/editor_pages/build_plugins/plugins/plugin.svg` | `direct` | Wired from former `extension-outline`; exact plugin domain role. |

## Deferred Secondary Surfaces

Material component lab `.zui` icon examples are deferred because they are component showcase/prototype surfaces rather than production Editor shell chrome. They should be revisited in a later Material-lab mapping pass after the production template wiring policy is accepted.

Observed Material lab examples:

| Surface | Observed icon usage | Deferral note |
| --- | --- | --- |
| `zircon_editor/assets/ui/editor/material_components/material_floating_action_button.zui` | `IconButton` samples use `icon = "add"` and `icon = "edit"`; the extended `Button` sample also uses `icon = "add"`. | Material FAB examples are generic component states, not Editor page/function roles. |
| `zircon_editor/assets/ui/editor/material_components/material_speed_dial.zui` | `IconButton` sample exists without a concrete `icon` or Ionicons `value` prop. | Component behavior sample, no production icon role to map. |
| `zircon_editor/assets/ui/editor/material_components/material_icons.zui` | `Icon` sample exists without a concrete `icon` or Ionicons `value` prop. | Icon component showcase, no production icon role to map. |

Other non-production `.zui` showcase examples observed during validation, also deferred:

| Surface | Observed icon usage | Deferral note |
| --- | --- | --- |
| `zircon_editor/assets/ui/editor/components/showcase_input_section.zui` | `IconButton` sample uses `icon = "add-outline"`. | Generic input showcase, not production shell chrome. |
| `zircon_editor/assets/ui/editor/components/showcase_visual_section.zui` | `Icon` sample uses `icon = "options-outline"` and `value = "ionicons/options-outline.svg"`. | Generic visual showcase, not production shell chrome. |

Demo and showcase `.v2.ui.toml` files are also outside the first production mapping scope unless a future plan elects to map demos. They are demo surfaces rather than builtin production shell templates for this milestone.

Observed deferred demo `.v2.ui.toml` usages:

| Surface | Observed icon usage | Deferral note |
| --- | --- | --- |
| `zircon_editor/assets/ui/editor/material_demo_window.v2.ui.toml:68` | `save-outline` | Demo window chrome sample, not production shell mapping scope. |
| `zircon_editor/assets/ui/editor/fyrox_panel_demo_window.v2.ui.toml:49` | `move` | Fyrox panel demo control sample, not production shell mapping scope. |

## Validation Notes

- Production mapping scope was searched for `component = "Icon"`, `component = "IconButton"`, `icon = "`, and `value = "ionicons/`.
- Every production template icon usage found in the mapping scope is represented in the table above as `direct`, `near`, or `gap`.
- `gap` rows are intentional blockers for future wiring, not current implementation failures.
- Milestone 1 wired all 39 `direct` and `near` rows in production templates; Milestone 2 updates documentation only.
