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
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
  - docs/zircon_editor/assets/editor-page-function-svg-resources.md
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
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
  - docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
  - docs/zircon_editor/assets/editor-page-function-svg-resources.md
  - docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md
plan_sources:
  - user: 2026-05-23 wire production Editor templates to accepted editor_pages icon mappings
  - user: 2026-05-25 complete live Editor visual rendering and 16px readability validation for wired editor_pages icons
  - docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
tests:
  - template wiring validation: 39 direct/near rows use expected editor_pages paths, referenced SVGs exist, and 7 gaps remain unchanged
  - asset inventory: 204 SVG files under zircon_editor/assets/icons/editor_pages with page groups A 61, B 54, C 89
  - SVG contract scans: no forbidden constructs and no non-ASCII bytes under zircon_editor/assets/icons/editor_pages
  - cargo test -p zircon_editor --lib repository_assets --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture
  - cargo test -p zircon_editor editor_pages_template_icons_have_readable_16px_raster_footprints --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --ignored --nocapture
  - cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation
doc_type: milestone-detail
---

# Editor Pages Template Icon Wiring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire production Editor `*.v2.ui.toml` icon metadata to the validated `editor_pages` SVG pack for all accepted `direct` and `near` mappings, while leaving current `gap` rows unchanged.

**Architecture:** This is an editor-authoring asset wiring pass owned by `zircon_editor`. It changes production template metadata only, not retained-host Rust projection, runtime UI contracts, icon resolver code, atlas logic, demo/showcase surfaces, or SVG inventory. The mapping document becomes the source of truth for what is wired now and what remains deferred.

**Tech Stack:** TOML UI v2 templates, Markdown documentation, Rust template projection tests, PowerShell validation scripts.

---

## File Structure

- Modify production template assets:
  - `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/console_body.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/hierarchy_body.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/animation_graph_body.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/animation_sequence_body.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/performance_timeline_body.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/build_export_desktop_body.v2.ui.toml`
  - `zircon_editor/assets/ui/editor/host/module_plugins_body.v2.ui.toml`
- Modify docs:
  - `docs/zircon_editor/assets/editor-page-function-icon-template-map.md`
  - `docs/zircon_editor/assets/editor-page-function-svg-resources.md`
  - `docs/zircon_editor/ui/retained_host/performance.md`
  - `docs/zircon_app/editor-host-entry.md`
  - `docs/zircon_app/plugins.md`
  - `docs/runtime-plugins/profile-selection.md`
- Reference only:
  - `docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md`
  - `docs/superpowers/specs/2026-05-21-editor-svg-polish-ui-mapping-design.md`
  - `docs/superpowers/plans/2026-05-21-editor-svg-polish-ui-mapping.md`
- Do not modify:
  - `zircon_editor/assets/icons/editor_pages/**/*.svg`; the inventory must remain 204.
  - `.zui` component lab files, demo/showcase `.v2.ui.toml` files, icon registries, atlas code, or existing icon packs.
- Rust source was out of scope for the initial template wiring milestone. The later visual validation addendum made only scoped Rust changes needed to prove retained-host 16px readability and build the live editor-host binary.

## Shared Wiring Table

Use this table as the exact replacement source. Preserve every non-icon property in the target TOML rows.

| Template | Control id | Replace prop(s) with | Confidence |
| --- | --- | --- | --- |
| `host/workbench_shell.v2.ui.toml` | `OpenProject` | `icon = "editor_pages/workbench/menu/open-project.svg"` | `direct` |
| `host/workbench_shell.v2.ui.toml` | `SaveProject` | `icon = "editor_pages/workbench/menu/save-all.svg"` | `direct` |
| `host/workbench_shell.v2.ui.toml` | `ResetLayout` | `icon = "editor_pages/workbench/dock/reset-layout.svg"` | `direct` |
| `host/workbench_shell.v2.ui.toml` | `AssetsToggle` | `icon = "editor_pages/asset_browser/navigation/folder.svg"` | `direct` |
| `host/workbench_shell.v2.ui.toml` | `HierarchyToggle` | `icon = "editor_pages/hierarchy/entity/scene.svg"` | `direct` |
| `host/workbench_shell.v2.ui.toml` | `ConsoleToggle` | `icon = "editor_pages/console_profiler/logs/log-info.svg"` | `near` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetTool` | `icon = "editor_pages/scene_viewport/tools/universal-transform.svg"` | `near` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetDisplayMode` | `icon = "editor_pages/scene_viewport/display/lit.svg"` | `near` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetGridMode` | `icon = "editor_pages/scene_viewport/display/grid-overlay.svg"` | `direct` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetTranslateSnap` | `icon = "editor_pages/scene_viewport/snapping/grid-snap.svg"` | `near` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetRotateSnapDegrees` | `icon = "editor_pages/scene_viewport/snapping/angle-snap.svg"` | `direct` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetScaleSnap` | `icon = "editor_pages/scene_viewport/snapping/scale-snap.svg"` | `direct` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetPreviewLighting` | `icon = "editor_pages/scene_viewport/display/lit.svg"` | `near` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetGizmosEnabled` | `icon = "editor_pages/scene_viewport/display/gizmo-visibility.svg"` | `direct` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `FrameSelection` | `icon = "editor_pages/scene_viewport/camera/frame-selection.svg"` | `direct` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `EnterPlayMode` | `icon = "editor_pages/scene_viewport/play/play.svg"` | `direct` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `ExitPlayMode` | `icon = "editor_pages/scene_viewport/play/stop.svg"` | `direct` |
| `host/scene_viewport_toolbar.v2.ui.toml` | `SetProjectionMode` | `icon = "editor_pages/scene_viewport/camera/perspective.svg"` | `near` |
| `host/asset_surface_controls.v2.ui.toml` | `SelectFolder` | `icon = "editor_pages/asset_browser/navigation/folder.svg"` | `direct` |
| `host/asset_surface_controls.v2.ui.toml` | `SetUtilityTab` | `icon = "editor_pages/asset_browser/import_pipeline/import-settings.svg"` | `near` |
| `host/asset_surface_controls.v2.ui.toml` | `ActivateReference` | `icon = "editor_pages/asset_browser/references/reference.svg"` | `direct` |
| `host/asset_surface_controls.v2.ui.toml` | `OpenAssetBrowser` | `icon = "editor_pages/asset_browser/navigation/folder.svg"` | `near` |
| `host/asset_surface_controls.v2.ui.toml` | `LocateSelectedAsset` | `icon = "editor_pages/asset_browser/navigation/search.svg"` | `near` |
| `host/asset_surface_controls.v2.ui.toml` | `ImportModel` | `icon = "editor_pages/asset_browser/import_pipeline/import.svg"` | `direct` |
| `host/startup_welcome_controls.v2.ui.toml` | `OpenExistingProject` | `icon = "editor_pages/workbench/menu/open-project.svg"` | `direct` |
| `host/startup_welcome_controls.v2.ui.toml` | `OpenRecentProject` | `icon = "editor_pages/asset_browser/navigation/recent.svg"` | `near` |
| `host/startup_welcome_controls.v2.ui.toml` | `RemoveRecentProject` | `icon = "editor_pages/workbench/tabs/close-tab.svg"` | `near` |
| `workbench_activity_rail.v2.ui.toml` | `ActivityRailButtonIcon0` | `icon = "editor_pages/asset_browser/navigation/folder.svg"`; `value = "editor_pages/asset_browser/navigation/folder.svg"` | `direct` |
| `workbench_activity_rail.v2.ui.toml` | `ActivityRailButtonIcon1` | `icon = "editor_pages/hierarchy/entity/scene.svg"`; `value = "editor_pages/hierarchy/entity/scene.svg"` | `direct` |
| `workbench_dock_header.v2.ui.toml` | `DockTabClose0` | `icon = "editor_pages/workbench/tabs/close-tab.svg"`; `value = "editor_pages/workbench/tabs/close-tab.svg"` | `direct` |
| `workbench_dock_header.v2.ui.toml` | `DockTabClose1` | `icon = "editor_pages/workbench/tabs/close-tab.svg"`; `value = "editor_pages/workbench/tabs/close-tab.svg"` | `direct` |
| `host/console_body.v2.ui.toml` | `FocusConsole` | `icon = "editor_pages/console_profiler/logs/log-info.svg"` | `near` |
| `host/hierarchy_body.v2.ui.toml` | `SelectRoot` | `icon = "editor_pages/hierarchy/entity/scene.svg"` | `direct` |
| `host/animation_graph_body.v2.ui.toml` | `AddNode` | `icon = "editor_pages/graph_editor/nodes/state-node.svg"` | `near` |
| `host/animation_sequence_body.v2.ui.toml` | `ScrubTimeline` | `icon = "editor_pages/animation_timeline/transport/timeline-play.svg"` | `near` |
| `host/performance_timeline_body.v2.ui.toml` | `RefreshTimelineSnapshot` | `icon = "editor_pages/console_profiler/profiling/frame-time.svg"` | `near` |
| `host/runtime_diagnostics_body.v2.ui.toml` | `FocusDiagnostics` | `icon = "editor_pages/console_profiler/diagnostics/watch.svg"` | `near` |
| `host/build_export_desktop_body.v2.ui.toml` | `FocusBuildExport` | `icon = "editor_pages/build_plugins/package/package.svg"` | `near` |
| `host/module_plugins_body.v2.ui.toml` | `FocusModulePlugins` | `icon = "editor_pages/build_plugins/plugins/plugin.svg"` | `direct` |

## Milestone 1: Template Wiring

### Goal

Update production Editor template icon metadata for all 39 `direct` and `near` rows without changing any layout, routing, component structure, Rust source, or `gap` row.

### In-Scope Behaviors

- `IconButton` rows use `icon = "editor_pages/...svg"` for accepted mappings.
- Static `Icon` rows with existing `value = "ionicons/..."` use matching `editor_pages/...svg` for both `icon` and `value`.
- The seven `gap` rows remain unchanged:
  - `SetTransformSpace`: `resize-outline`
  - `SetPreviewSkybox`: `cloud-outline`
  - `AlignView`: `navigate-outline`
  - `SelectItem`: `cube-outline`
  - `SetViewMode`: `list-outline`
  - `TriggerAction`: `flash-outline`
  - `CreateProject`: `add-circle-outline`
- Demo, showcase, and Material component lab files remain unchanged.

### Dependencies

- Approved design spec: `docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md`.
- Existing mapping doc: `docs/zircon_editor/assets/editor-page-function-icon-template-map.md`.
- Existing `editor_pages` inventory: 204 SVG files.

### Implementation Slices

- [x] Edit `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml` according to the shared wiring table.
- [x] Edit `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` according to the shared wiring table, leaving `SetTransformSpace`, `SetPreviewSkybox`, and `AlignView` unchanged.
- [x] Edit `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` according to the shared wiring table, leaving `SelectItem` and `SetViewMode` unchanged.
- [x] Edit `zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml` according to the shared wiring table, leaving `CreateProject` unchanged.
- [x] Edit `zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml` so `ActivityRailButtonIcon0` and `ActivityRailButtonIcon1` set both `icon` and `value` to matching `editor_pages/...svg` paths.
- [x] Edit `zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml` so `DockTabClose0` and `DockTabClose1` set both `icon` and `value` to `editor_pages/workbench/tabs/close-tab.svg`.
- [x] Edit the single-icon pane body templates listed in the shared table: `console_body`, `hierarchy_body`, `animation_graph_body`, `animation_sequence_body`, `performance_timeline_body`, `runtime_diagnostics_body`, `build_export_desktop_body`, and `module_plugins_body`.
- [x] Do not edit `zircon_editor/assets/ui/editor/host/pane_surface_controls.v2.ui.toml`; its only icon row is a `gap` row.

### Testing Stage: Template Wiring Gate

- [x] Run a PowerShell validation script that parses the edited template TOML files, confirms all expected `editor_pages/...svg` values are present, confirms all wired paths exist under `zircon_editor/assets/icons`, and confirms the seven `gap` rows still have their original icon values.
- [x] Run `cargo test -p zircon_editor --lib repository_assets --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture` to verify repository host template assets still load/project; closeout rerun passed with 1 test, 0 failures, and 1510 filtered.
- [x] Diagnose the earlier cargo blocker as unrelated material-editor compile errors, not TOML syntax, template compilation, or asset lookup; a later rerun passed after the current workspace state no longer had that blocker, with no edited-template correction indicated by the static wiring gate.
- [x] Record the validation output in the final report and in the relevant docs updated by Milestone 2.

### Exit Evidence

- 39 `direct`/`near` template rows are wired.
- 7 `gap` rows remain unchanged.
- Scoped editor template test passed after an earlier unrelated material-editor compile blocker cleared outside this wiring pass; static wiring and asset checks also passed.
- No Rust source, `.zui`, demo/showcase `.v2.ui.toml`, or SVG file is modified by this milestone.

## Milestone 2: Documentation Sync And Final Asset Gate

### Goal

Update docs to describe actual template wiring state, then rerun whole-pack and scoped status validation.

### In-Scope Behaviors

- `docs/zircon_editor/assets/editor-page-function-icon-template-map.md` states that `direct` and `near` rows are now wired in production templates.
- The mapping table keeps the `gap` rows and describes them as intentionally unwired.
- `docs/zircon_editor/assets/editor-page-function-svg-resources.md` records the template wiring follow-up and validation evidence.
- `docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md` remains the design source; update only if implementation evidence changes a design assertion.

### Dependencies

- Milestone 1 template wiring gate passes or records a concrete blocker that requires a scoped implementation correction.

### Implementation Slices

- [x] Update the frontmatter in `docs/zircon_editor/assets/editor-page-function-icon-template-map.md` if needed so `implementation_files` includes the edited production templates and the new wiring spec/plan.
- [x] Update the mapping introduction to distinguish current wired rows from remaining gap rows.
- [x] Add or update a wiring status summary with counts: `wired direct = 22`, `wired near = 17`, `unwired gap = 7`, total `46` production rows.
- [x] Update individual `direct` and `near` rows if their `Current icon/value` column should now reflect the new `editor_pages/...svg` template value after wiring. Keep enough note text to preserve the original mapping rationale where useful.
- [x] Update `docs/zircon_editor/assets/editor-page-function-svg-resources.md` with a `Template Wiring Follow-Up` note, the new plan/spec source, validation commands, and the unchanged 204 inventory.
- [x] Do not update `docs/zircon_editor/assets/icon-resource-audit.md` because this plan preserves SVG paths and count.

### Testing Stage: Final Wiring And Asset Gate

- [x] Re-run the PowerShell template wiring validation from Milestone 1.
- [x] Count all SVGs under `zircon_editor/assets/icons/editor_pages` and confirm `204`.
- [x] Confirm page-group distribution remains A `61`, B `54`, C `89`.
- [x] Scan `editor_pages` SVG files for forbidden constructs: `<image`, `href=`, `<use`, `<symbol`, `<script`, `<style`, `style=`, `class=`, `@font`, `font-family`, and `url(http`.
- [x] Scan `editor_pages` SVG files for non-ASCII bytes.
- [x] Confirm `docs/zircon_editor/assets/editor-page-function-icon-template-map.md`, `docs/zircon_editor/assets/editor-page-function-svg-resources.md`, and `docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md` have machine-readable headers with `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type`.
- [x] Run `git status --short -- zircon_editor/assets/ui/editor zircon_editor/assets/icons/editor_pages docs/zircon_editor/assets/editor-page-function-icon-template-map.md docs/zircon_editor/assets/editor-page-function-svg-resources.md docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md` and review scope: intended template/docs/icon-polish files are dirty, while unrelated pre-existing `material_components/*.zui` files also appear under the broad UI path and are out of scope/not modified by this wiring pass.

### Exit Evidence

- Final validation confirms all 39 wired rows use existing `editor_pages` paths.
- Final validation confirms all 7 gap rows remain unchanged.
- Final validation confirms 204 SVG files and A/B/C distribution remains stable.
- Documentation states current wired/deferred status and records validation evidence.

## Post-Wiring Visual Validation Addendum

### Goal

Close the visual-risk note from the design spec by proving the wired `editor_pages` icons render through retained-host paths at 16 px and by capturing a live Editor window after the editor-host binary builds.

### Implementation Slices

- [x] Add retained-host painter coverage in `zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs` for the unique wired `editor_pages` template icon paths at 16 x 16 px.
- [x] Keep SVG inventory unchanged at 204 files and preserve all 7 `gap` rows.
- [x] Re-export the manifest-specific runtime-profile helper APIs from `zircon_runtime/src/lib.rs` so `zircon_app` can keep using the runtime crate root for provider-aware profile selection.
- [x] Keep the optional project manifest reusable in `zircon_app/src/entry/builtin_modules.rs` when the feature-registration resolver path needs the profile fallback manifest and later feature dependency checks still need the original optional manifest.
- [x] Run the ignored retained-host screenshot gate and inspect generated PNG dimensions/bytes.
- [x] Build the live editor-host binary and capture `target/visual-layout/editor-live-window-900x620.png`.
- [x] Sync docs with the retained-host, app/runtime, screenshot, and live capture evidence.
- [x] Rerun the focused closeout validation after the doc sync.

### Evidence Already Captured

- `cargo test -p zircon_runtime --lib scene::tests::ecs_system_query_cache --locked --offline --message-format short --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --test-threads=1 --nocapture` passed with 2 tests, 0 failures, and 2025 filtered, confirming the earlier ECS query-cache compile blocker was clear before editor closeout validation.
- `cargo test -p zircon_editor editor_pages_template_icons_have_readable_16px_raster_footprints --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture` passed on the closeout rerun with 1 test, 0 failures, and 1510 filtered; it printed `ICON_16PX_READABILITY` evidence for each unique wired icon.
- `cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --ignored --nocapture` passed in the earlier visual pass and wrote retained-host PNGs under `target/visual-layout`.
- `cargo test -p zircon_editor --lib repository_assets --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture` passed on the closeout rerun with 1 test, 0 failures, and 1510 filtered.
- `cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation` passed on the closeout rerun.
- Live capture wrote `target/visual-layout/editor-live-window-900x620.png`; the actual captured PNG was `1296 x 759` and `86492` bytes.

## Self-Review

- Spec coverage: Milestone 1 implements template-only wiring for `direct` and `near` rows while preserving gap rows; Milestone 2 synchronizes docs and performs final validation.
- Scope check: the template wiring milestones do not modify retained-host resolver behavior, atlas logic, demo/showcase surfaces, `.zui` files, or SVG inventory. The visual validation addendum modifies Rust only for retained-host readability coverage and app/runtime build-boundary fixes needed to compile and capture the live Editor.
- Placeholder scan: no unresolved placeholders are present.
- Type/path consistency: every path in the shared wiring table uses the approved `editor_pages/...svg` form and corresponds to an existing `zircon_editor/assets/icons/editor_pages/**` SVG path from the mapping document.
