---
related_code:
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/template_runtime/harness.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/asset_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/inspector_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/pane_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/welcome_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
implementation_files:
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/template_runtime/harness.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/asset_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/inspector_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/pane_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/welcome_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
plan_sources:
  - .codex/plans/Zircon Editor Runtime UI Rust-Owned Retained Host 重构计划.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
tests:
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/host_model.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/template_runtime/dual_host_parity.rs
  - zircon_editor/src/tests/host/template_runtime/viewport_toolbar.rs
  - zircon_editor/src/tests/host/template_runtime/asset_surface.rs
  - zircon_editor/src/tests/host/template_runtime/inspector_surface.rs
  - zircon_editor/src/tests/host/template_runtime/pane_surface_controls.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - cargo check -p zircon_editor --lib --locked --message-format=short
doc_type: module-detail
---

# Editor Template Runtime Retained Host Cutover

## Purpose

This document records the completed cutover from the old compatibility migration to the current retained template runtime. The file name is historical; the active behavior is no longer a compatibility path. Editor UI templates are `.ui.toml` documents compiled into shared UI surfaces and consumed by Rust-owned retained host bridges.

## Current Template Path

`EditorUiHostRuntime` is the editor-side runtime for builtin host templates and activity-window templates. It loads `.ui.toml` documents, registers component descriptors and stable bindings, builds shared `UiSurface` values, and projects host-neutral node lists for retained host consumers.

`RetainedUiHostAdapter` is the current adapter for host projections. It classifies nodes into retained host component kinds, preserves arranged frames and clip frames, maps TOML attributes into typed retained values, exposes component role and validation metadata, and carries binding routes forward as retained route projections.

The active adapter types are `RetainedUiHostProjection`, `RetainedUiHostNodeModel`, `RetainedUiHostComponentKind`, `RetainedUiHostRouteProjection`, and `RetainedUiHostValue`.

## Builtin Host Documents

Builtin host documents are source-controlled assets under `zircon_editor/assets/ui/editor/host`. The root host document identity is generic host-window authority even when the backing file remains named `workbench_shell.ui.toml`.

Important builtin documents include:

- `workbench_shell.ui.toml` for the root editor host shell
- `workbench_drawer_source.ui.toml` for drawer-source frame authority
- `floating_window_source.ui.toml` for floating-window default and clamp source frames
- `scene_viewport_toolbar.ui.toml` for viewport toolbar controls
- `asset_surface_controls.ui.toml` for asset browser and activity asset controls
- `inspector_surface_controls.ui.toml` for inspector actions and editable fields
- `pane_surface_controls.ui.toml` for generic pane actions
- `startup_welcome_controls.ui.toml` for welcome page actions

These assets are the current business UI structure. Rust bridges consume their shared-surface frames and stable control ids; no generated source tree owns the structure.

## Retained Template Bridges

`retained_host::callback_dispatch::template_bridge` contains the host bridge owners that connect compiled template authority to editor events. Folder roots stay structural; bridge behavior lives in child files such as `bridge.rs`, `source_frames.rs`, `host_projection.rs`, and `layout.rs`.

The active bridge families are:

- `workbench` for root host window frames and workbench binding lookup
- `workbench_drawer_source` for drawer shell/header/content source frames
- `floating_window_source` for independent floating-window source frames
- `viewport_toolbar` for viewport toolbar control ids and route binding
- `asset_surface` for asset tree/content/reference and utility controls
- `inspector_surface` for inspector apply/delete/draft controls
- `pane_surface` for generic pane actions
- `welcome_surface` for welcome project actions and recent-project controls

The runtime path is consistent across these bridges: control id and event kind are resolved against the compiled template projection, mapped to a stable editor binding, then dispatched through `EditorEventRuntime` or the relevant retained host event reducer.

## Projection And Presentation Boundary

`retained_host::ui::apply_presentation` is the presentation entry point that converts workbench state into retained host-contract data. It consumes template bridge frame bundles, floating-window projection bundles, component showcase runtime overlays, and editor pane view data.

`retained_host::ui::template_node_conversion` and `retained_host::ui::pane_data_conversion` are the retained DTO conversion layer. Their conversion helpers use `to_host_contract_*` names because the output is Rust-owned host-contract data. They must not reintroduce old generated DTO conversion helpers.

## Removed Compatibility Surface

The older migration path used Slint host projections and generated source DTOs while `.ui.toml` templates were being introduced. That path has been hard-cut over. Current production and tests must not restore `SlintUiProjection`, generated include modules, `slint_build` seams, `slint_host` owner paths, or `workbench_slint*` test names as active authorities.

Historical references to Slint can remain in plans or deletion-guard prose, but they do not define current implementation. If a source, test, doc header, or validation command needs an owner name, use `retained_host`, `RetainedUiHost*`, `.ui.toml`, and `host_contract`.

## Validation

Current validation should focus on:

- template-runtime tests for builtin document identity, shared surface building, retained host model projection, route registration, and dual-host parity
- retained host tests for each template bridge family and pointer bridge family
- boundary tests that assert host template assets remain `.ui.toml` authority and active editor UI source trees contain no deleted generated files
- `cargo check -p zircon_editor --lib --locked --message-format=short`

Workspace-level validation remains the final milestone gate after unrelated active workstream blockers are either fixed by their owning sessions or classified as out of scope.
