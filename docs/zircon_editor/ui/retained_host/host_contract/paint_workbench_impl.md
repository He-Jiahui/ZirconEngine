---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/floating_windows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/panel_header.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/rail.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus/bar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/content.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/hierarchy.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/viewport.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/root_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/scene_layers.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/skeleton.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome/main_column.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome/recent_projects.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry.rs
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/floating_windows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/panel_header.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/rail.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/docks/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus/bar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/menus/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/content.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/hierarchy.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/viewport.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/root_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/scene_layers.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/skeleton.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome/main_column.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome/recent_projects.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/welcome/style.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-17 continue editor UI architecture implementation
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - workbench welcome ownership scan
  - workbench welcome subtree ownership scan
  - workbench menu ownership scan
  - workbench native pane ownership scan
  - workbench dock/pane ownership scan
  - workbench dock subtree ownership scan
  - workbench root orchestration subtree ownership scan
  - workbench menu subtree ownership scan
  - workbench native pane subtree ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-overview
---

# Paint Workbench Implementation

`paint_workbench.rs` is the neutral retained-host entry that decides whether to draw the componentized Workbench surface or the legacy fallback. `paint_workbench_impl.rs` now owns the concrete window-level orchestration only: module declarations, public draw entries, shared Workbench painter palette constants, and shared fallback text selection. The root-frame fallback, legacy skeleton, scene layer, concrete dock, pane, menu, native pane, and Welcome bodies live in child modules while the M3 hard cutover continues.

`paint_workbench_impl/root_frames.rs` owns `RootFrames`, root-frame fallback selection, top-bar/status/center/left/right/bottom/document/viewport fallback geometry, and the zero-origin helper shared by template overlay drawing. `paint_workbench_impl/skeleton.rs` owns legacy root skeleton fills, borders, separators, project marker, debug refresh-rate marker, viewport label, and status secondary label. `paint_workbench_impl/scene_layers.rs` owns host-scene paint order, componentized Workbench window drawing, root template overlay, resize layer, close prompt layer, menu layer, dock layer, floating layer, and status-bar template-node layer.

`paint_workbench_impl/docks.rs` owns the legacy dock shell entry path: side/document/bottom dock shells, region backgrounds, rail/panel frame resolution, and calls into the focused dock children. `docks/panel_header.rs` owns header fill/template/separator paint. `docks/pane.rs` owns pane background/body orchestration, template-node dispatch, native pane dispatch, debug overlay, and fallback labels. `docks/viewport_toolbar.rs` owns Scene/Game viewport toolbar chrome. `docks/rail.rs` owns the active activity-rail marker. `docks/floating_windows.rs` owns floating layer iteration, floating-window shell/header/body layout, and floating pane dispatch. The parent module calls `docks::draw_side_dock(...)`, `docks::draw_document_dock(...)`, `docks::draw_bottom_dock(...)`, and `docks::draw_floating_layer(...)` from the host-scene paint orchestration.

`paint_workbench_impl/menus.rs` is now the structural entry for Workbench menu paint. It keeps the two host-scene entry points, `draw_menu_bar_labels(...)` and `draw_open_menu_popup(...)`, and delegates all behavior to the `menus/` subtree. `menus/bar.rs` owns top-bar label paint, active-menu highlighting, clipped label drawing, menu frame scroll adjustment, and menu-bar separators. `menus/popup.rs` owns open menu popup paint, custom template popup dispatch, submenu popup chaining, branch traversal, and popup body/background/border paint. `menus/rows.rs` owns fallback popup row text, shortcut, disabled, and hover paint. `menus/geometry.rs` owns row frame math, popup height math, top-level popup viewport constraints, submenu side-placement constraints, and scrolled menu-frame adjustment.

`paint_workbench_impl/native_panes.rs` is now the structural entry for retained native pane paint. It keeps `draw_viewport_image(...)`, `draw_native_pane_content(...)`, and `draw_pane_debug_overlay(...)` as the dock-facing API and delegates details to the `native_panes/` subtree. `native_panes/viewport.rs` owns Scene/Game viewport image validation and RGBA image draw dispatch. `native_panes/diagnostics.rs` owns RuntimeDiagnostics overlay primitive collection and debug-reflector overlay dispatch. `native_panes/content.rs` owns native pane kind routing across Welcome, Hierarchy, Assets, and AssetBrowser. `native_panes/hierarchy.rs` owns Hierarchy row paint, selection/hover colors, indentation, scrolling, and template-derived viewport fallback. `native_panes/assets.rs` owns Asset activity and AssetBrowser tree hover overlays, row-control matching, row frame translation, scroll offsets, and hover border paint.

`paint_workbench_impl/welcome.rs` is now the structural entry for the Welcome pane native content path. It decides whether the pane has enough data to draw, resolves the outer/recent/main panels, fills panel backgrounds, and delegates body details. `welcome/layout.rs` owns template frame lookup, fallback insets, content width constraints, and shared row metrics. `welcome/style.rs` owns Welcome palette aliases and action colors. `welcome/main_column.rs` owns the hero/status/new-project form/preview/validation/action column. `welcome/recent_projects.rs` owns the recent-project header, empty state, list rows, invalid-row border, and row status labels.

The 2026-06-18 welcome split reduced `paint_workbench_impl.rs` from about 1957 lines to 1428 and created `paint_workbench_impl/welcome.rs` at 564 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a workbench welcome ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current feature-first cadence.

The 2026-06-18 menu split reduced `paint_workbench_impl.rs` to 1159 lines and created `paint_workbench_impl/menus.rs` at 287 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a workbench menu ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 native pane split reduced `paint_workbench_impl.rs` to 938 lines and created `paint_workbench_impl/native_panes.rs` at 233 lines, with `menus.rs` at 287 lines and `welcome.rs` at 564 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a workbench native pane ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 dock/pane split reduced `paint_workbench_impl.rs` to 434 lines and created `paint_workbench_impl/docks.rs` at 516 lines, with `native_panes.rs` at 233 lines, `menus.rs` at 287 lines, and `welcome.rs` at 564 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a workbench dock/pane ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 Welcome subtree split reduced the Welcome entry file from 564 lines to 60 lines. The new child owners are `welcome/layout.rs` at 36 lines, `welcome/style.rs` at 13 lines, `welcome/main_column.rs` at 360 lines, and `welcome/recent_projects.rs` at 135 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a Welcome subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 dock subtree split reduced `paint_workbench_impl/docks.rs` from 516 lines to 185 lines. The new child owners are `docks/panel_header.rs` at 34 lines, `docks/pane.rs` at 182 lines, `docks/viewport_toolbar.rs` at 37 lines, `docks/rail.rs` at 28 lines, and `docks/floating_windows.rs` at 86 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a dock subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 root orchestration subtree split reduced `paint_workbench_impl.rs` from 412 lines to 66 lines. The new child owners are `root_frames.rs` at 99 lines, `skeleton.rs` at 94 lines, and `scene_layers.rs` at 177 lines. Validation used `cargo fmt -p zircon_editor --check`, a workbench root orchestration subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current feature-first cadence.

The 2026-06-18 menu subtree split reduced `paint_workbench_impl/menus.rs` from 275 lines to 18 lines. The new child owners are `menus/geometry.rs` at 110 lines, `menus/popup.rs` at 88 lines, `menus/rows.rs` at 48 lines, and `menus/bar.rs` at 44 lines. Validation used `cargo fmt -p zircon_editor --check`, a workbench menu subtree ownership scan, a touched-file trailing whitespace scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current feature-first cadence.

The 2026-06-18 native pane subtree split reduced `paint_workbench_impl/native_panes.rs` from 222 lines to 35 lines. The new child owners are `native_panes/assets.rs` at 109 lines, `native_panes/hierarchy.rs` at 85 lines, `native_panes/viewport.rs` at 26 lines, `native_panes/content.rs` at 23 lines, and `native_panes/diagnostics.rs` at 17 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a native pane subtree ownership scan, a touched-file trailing whitespace scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current feature-first cadence.
