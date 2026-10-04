---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/animation.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/basic.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/build_export.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/hierarchy.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/inspector.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/module_plugins.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/performance_timeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/viewport.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/animation.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/basic.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/build_export.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/hierarchy.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/inspector.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/module_plugins.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/performance_timeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/viewport.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-21 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - host_contract data panes DTO subtree ownership scan
  - scoped whitespace scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-21 attempted before this adjacent split; timed out after 300 seconds before actionable editor diagnostics)
doc_type: module-detail
---

# Pane Host Data

`data/panes.rs` is the retained-host pane DTO entry. It no longer owns every pane record directly; it declares focused child modules and re-exports their data types through the existing `data::*` boundary.

## Purpose

The pane data subtree describes editor pane presentation state after runtime/template projection has produced retained host records. These types are data contracts only. They should not own projection, paint, hit testing, native input routing, persistence, or runtime scheduling behavior.

## Module Ownership

`panes/pane.rs` owns the aggregate `PaneData` record that ties pane chrome metadata, frame state, and individual pane-family payloads together.

`panes/viewport.rs` owns scene viewport chrome state, including tool/mode labels, snap labels, preview toggles, and the optional toolbar surface frame.

`panes/hierarchy.rs` owns scene hierarchy row data and hierarchy pane nodes. `panes/inspector.rs` owns inspector pane fields. `panes/animation.rs` owns animation editor timeline and selection labels.

`panes/runtime_diagnostics.rs` owns runtime diagnostics pane data and debug overlay primitive records. It keeps the non-derived default for `UiDebugOverlayPrimitiveKind::SelectedFrame` inside the diagnostics owner instead of the root data file.

`panes/performance_timeline.rs` owns performance timeline row families, capture controls, and the aggregate timeline pane record.

`panes/module_plugins.rs` owns module/plugin status rows and the module plugins pane aggregate. `panes/build_export.rs` owns BuildExport target rows and pane diagnostics. `panes/basic.rs` owns simple pane families whose records are just node lists or short labels: console, assets activity, asset browser, project overview, generated bottom pane, and project overview metadata.

## Behavior Model

The root file stays structural so callers can keep importing pane records through `host_contract::data::*`. Each child file groups DTOs by pane domain, which keeps future schema additions local and prevents unrelated fields from accumulating in one declaration bucket.

The split is deliberately data-only. If a field needs behavior, that behavior belongs in the existing owners that consume the data: pane data conversion, retained app state, template-node painting, surface hit testing, native pointer/keyboard routing, or runtime UI extraction.

## Test Coverage

This slice was validated with `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an ownership scan confirming the root pane file no longer owns concrete DTO declarations, a scoped whitespace scan, and scoped `git diff --check`. The package-level editor Cargo check was already attempted in this 2026-06-21 M3.S2 pass and timed out after 300 seconds without actionable editor diagnostics; full Cargo tests remain deferred to the milestone testing stage per the user request.
