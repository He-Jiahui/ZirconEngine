---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/component_showcase.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/ui_asset.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/component_showcase.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/ui_asset.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app callback-wiring pane-surface ownership scan
  - app callback-wiring viewport/UI Asset callback ownership scan
  - app callback-wiring Component Showcase callback ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Callback Wiring

`app/callback_wiring.rs` owns the top-level retained host callback installation entry. It wires `UiHostContext` callbacks for frame ticks, unhandled keyboard input, close prompt actions, menu pointer events, activity rail clicks, tab/header clicks, drag capture, and resize capture. It also keeps `dispatch_with_callback_source(...)`, the shared helper that preserves the source host window while dispatching callbacks into `RetainedEditorHost`.

## Pane Surface Callbacks

`app/callback_wiring/pane_surface.rs` owns `PaneSurfaceHostContext` callback registration. It wires Welcome, Hierarchy, Console, Inspector, pane surface controls, and Workbench context menu callbacks directly, then delegates asset, Component Showcase, viewport, and UI Asset editor callback groups to child modules.

The child module uses the parent `dispatch_with_callback_source(...)` helper for callbacks that need source-window attribution. Direct host-shell callbacks that are already scoped to the root window stay in `callback_wiring.rs`.

## Asset Callbacks

`app/callback_wiring/pane_surface/assets.rs` owns the asset-related pane-surface callback registration set. It wires mesh import path editing, asset control changes/clicks, asset tree pointer events, asset content pointer events, asset reference list pointer events, and browser asset details scrolling.

## Component Showcase Callbacks

`app/callback_wiring/pane_surface/component_showcase.rs` owns Component Showcase pane-surface callback registration. It wires activate, drag-delta, edit, context request, and option selection callbacks into the retained host Component Showcase action owner while preserving callback source-window attribution.

## Viewport Callbacks

`app/callback_wiring/pane_surface/viewport.rs` owns viewport pane-surface callback registration. It wires viewport pointer events and viewport toolbar pointer clicks into the retained host viewport action owners while preserving callback source-window attribution.

## UI Asset Callbacks

`app/callback_wiring/pane_surface/ui_asset.rs` owns UI Asset editor pane-surface callback registration. It wires UI Asset top-level actions, detail events, and collection events into the retained host UI Asset editor action owners while preserving callback source-window attribution.

## Boundary Rules

- Keep `UiHostContext` callback installation and callback source helper ownership in `app/callback_wiring.rs`.
- Keep general non-asset `PaneSurfaceHostContext` callback installation in `app/callback_wiring/pane_surface.rs`.
- Keep mesh import and asset pane callback registration in `app/callback_wiring/pane_surface/assets.rs`.
- Keep Component Showcase callback registration in `app/callback_wiring/pane_surface/component_showcase.rs`.
- Keep viewport pointer and toolbar callback registration in `app/callback_wiring/pane_surface/viewport.rs`.
- Keep UI Asset editor action/detail/collection callback registration in `app/callback_wiring/pane_surface/ui_asset.rs`.
- Keep actual pane/action execution in feature modules such as `pane_surface_actions`, asset handlers, viewport handlers, and UI Asset editor handlers; callback wiring should only adapt callback arguments and forward into those owners.
- Keep source-window attribution explicit for callbacks coming from pane surfaces.

## Validation Notes

The 2026-06-18 pane-surface callback split reduced `callback_wiring.rs` from 669 lines to 153 lines. `callback_wiring/pane_surface.rs` is 527 lines and owns the pane-surface callback registration set.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring pane-surface ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 asset callback split reduced `callback_wiring/pane_surface.rs` from 527 lines to 324 lines. `callback_wiring/pane_surface/assets.rs` is 216 lines and owns the mesh import, asset control, asset tree/content/reference, and browser asset details callback registration set. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring asset callback ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 viewport/UI Asset callback split reduced `callback_wiring/pane_surface.rs` from 324 lines to 245 lines. `callback_wiring/pane_surface/viewport.rs` is 43 lines and owns viewport pointer/toolbar callback registration; `callback_wiring/pane_surface/ui_asset.rs` is 59 lines and owns UI Asset action/detail/collection callback registration. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring viewport/UI Asset callback ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 Component Showcase callback split reduced `callback_wiring/pane_surface.rs` from 245 lines to 177 lines. `callback_wiring/pane_surface/component_showcase.rs` is 79 lines and owns Component Showcase activation, drag, edit, context, and option callback registration. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring Component Showcase callback ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.
