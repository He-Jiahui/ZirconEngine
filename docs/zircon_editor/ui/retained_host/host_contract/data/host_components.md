---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/common.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/docks.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/menus.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/window.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/common.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/docks.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/menus.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/window.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-21 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - host_contract data host-components DTO subtree ownership scan
  - scoped whitespace scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-21 attempted before this adjacent split; timed out after 300 seconds before actionable editor diagnostics)
doc_type: module-detail
---

# Host Component Data

`data/host_components.rs` is the retained-host DTO entry for window chrome, dock surfaces, tabs, menu chrome, floating windows, and shell metrics. It is now a structural module root that re-exports focused child owner files through the existing `data::*` boundary.

## Purpose

Host component data records describe the editor window shell after layout/projection has produced concrete chrome facts. They are DTOs only. They carry ids, labels, frames, tab models, menu rows, dock panes, splitter regions, floating-window metadata, and orchestration metrics consumed by painting, pointer routing, presentation, and runtime draw-list projection.

## Module Ownership

`host_components/common.rs` owns shared tab and frame records: `TabData`, `FrameRect`, `HostChromeControlFrameData`, and `HostChromeTabData`.

`host_components/window.rs` owns whole-window shell/layout/bootstrap/surface and tab-drag overlay records. This keeps high-level window composition separate from individual dock and menu chrome structures.

`host_components/docks.rs` owns resize-layer, side-dock, document-dock, and bottom-dock surface data. It depends on the pane DTO subtree for the active pane payload but does not own pane-family schemas.

`host_components/floating.rs` owns floating-window records, floating-window layer data, and the native floating-window surface record.

`host_components/menus.rs` owns menu item/menu models plus menu bar, host page chrome, and status bar DTOs.

`host_components/metrics.rs` owns shared surface metrics and orchestration offsets used by host window assembly.

## Behavior Model

The root file stays a module declaration and re-export surface. Child modules use sibling `data` owners for `PaneData` and `TemplatePaneNodeData` instead of importing through the broader `host_contract` root, so the DTO subtree remains locally wired and the root export stays a convenience boundary for outside callers.

The split preserves the existing public internal paths because `data/mod.rs` still re-exports `host_components::*`. No compatibility shim was added; the former mixed declaration file was replaced by the folder-backed owner tree.

## Test Coverage

This slice was validated with `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an ownership scan confirming `host_components.rs` no longer owns concrete DTO declarations, a scoped whitespace scan, and scoped `git diff --check`. The package-level editor Cargo check was already attempted in this 2026-06-21 M3.S2 pass and timed out after 300 seconds without actionable editor diagnostics; full Cargo tests remain deferred to the milestone testing stage per the user request.
