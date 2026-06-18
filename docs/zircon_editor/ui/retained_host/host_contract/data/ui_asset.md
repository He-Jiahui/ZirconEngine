---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/canvas.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/common.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/inspector.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/preview.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/source.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/canvas.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/common.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/inspector.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/preview.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/source.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset/style.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - UI Asset DTO subtree ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# UI Asset Host Data

`data/ui_asset.rs` is the retained-host DTO surface for the componentized UI Asset editor pane. It keeps the external `ui_asset::*` re-export boundary stable while the concrete data families live in focused child modules.

## Purpose

The UI Asset editor pane combines several authoring surfaces: a source panel, preview canvas/mock panel, style editor, inspector, palette drag affordance, runtime report, action policy, and pane header. These records are host-contract data only. They should not own projection logic, paint logic, callback routing, persistence, or runtime surface hit testing.

## Module Ownership

`ui_asset.rs` is now a structural module root. It declares child modules and re-exports their DTOs for existing `data::*` users.

`ui_asset/common.rs` owns shared list-selection data used by source outline, preview selectors, style selectors, inspector binding suggestions, collections, and runtime locale preview.

`ui_asset/canvas.rs` owns canvas-node geometry, slot-target previews, palette drag state, and preview-canvas dimensions/items.

`ui_asset/source.rs` owns source text panel and source-detail state, including selected line, cursor offset, selected excerpt, roundtrip status, and source outline.

`ui_asset/preview.rs` owns preview mock and preview panel data, including mock subject/property/nested/suggestion state plus schema and state-graph item lists.

`ui_asset/style.rs` owns theme-source, style-rule, matched-rule, declaration, token, state, and style-panel data.

`ui_asset/inspector.rs` owns inspector semantic, slot, layout, binding, widget property state, widget, and inspector-panel data.

`ui_asset/pane.rs` owns pane-level UI Asset records: header, action state, designer tool state, collection panels, runtime report, and the aggregate `UiAssetEditorPaneData` that ties template nodes and all UI Asset child panels together.

## Behavior Model

Every type in this subtree is a cloneable/defaultable data record. The records are passed through retained-host presentation and template-node projection. The subtree must remain data-only: child modules define schema shape and grouping, while behavior stays in pane data conversion, callback dispatch, template projection, paint, native pointer/keyboard routing, or runtime UI extraction modules.

`UiAssetEditorPaneData` remains the aggregate contract for the editor pane. It references `TemplatePaneNodeData` for the projected node tree and specific panel root nodes, then embeds child DTOs for header/actions/collections/source/preview/runtime report/designer tools/palette drag/style/inspector.

## Design And Rationale

The former single file mixed more than two dozen DTO declarations. Splitting by authoring subdomain makes future changes local: style editor fields no longer share a declaration file with inspector binding state, source panel details, or runtime report data.

The root keeps re-export compatibility because many host-contract callers import through `super::data::*`. This is a structural reorganization only; it does not create a legacy façade or duplicate data model.

## Edge Cases And Constraints

- Do not add projection behavior to these DTO files.
- Do not put paint or hit-test helpers into the UI Asset data subtree.
- Keep shared selector state in `common.rs` unless it becomes specific to one panel family.
- Keep the aggregate pane contract in `pane.rs`; child panel DTOs should not import the aggregate.
- New UI Asset authoring subdomains should get a new child file rather than growing the root module.

## Test Coverage

This slice was validated with `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a UI Asset DTO subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`. Full unit and integration test matrices remain deferred to the milestone testing stage per the active 08 plan and the user request to implement functionality first.

## Plan Sources

This module supports `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, specifically M3.S2 retained-host owner shrink work. The final architecture goal is still to move editor UI pixels toward runtime render extract and GPU command stream ownership while keeping editor-specific authoring data in `zircon_editor`.
