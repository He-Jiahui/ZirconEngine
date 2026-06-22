---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/drag.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/text_focus.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/drag.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/text_focus.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/resize.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-21 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - data host-interaction menu/drag/text-focus/pane/resize ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Host Interaction Data

`data/host_interaction.rs` is the retained-host interaction state entry. It now keeps only child
module declarations and stable re-exports for the state DTOs consumed by globals, window
presentation, native pointer routing, template hover overlays, and focused text painting.

## Ownership

`host_interaction/menu.rs` owns `HostMenuStateData`, including the `-1` closed/no-hover sentinels
used by menu damage and popup paint paths. `host_interaction/drag.rs` owns `HostDragStateData`,
which tracks active tab drag identity, pointer location, and source/target groups.

`host_interaction/text_focus.rs` owns `HostTextInputFocusData` and the narrow helper methods for
active focus detection plus edit/commit target resolution. `host_interaction/pane.rs` owns
`HostPaneInteractionStateData`, the pointer-only pane hover and scroll state used for Hierarchy,
Assets, AssetBrowser, and template hover overlays without rebuilding the full retained
presentation. `host_interaction/resize.rs` owns `HostResizeStateData`, the active host resize
capture payload.

## Behavior Model

The data remains plain retained-host state. `globals/state.rs` stores these DTOs in
`HostContractState`, `globals/ui_context.rs` exposes host-level get/set operations, native pointer
dispatch mutates the relevant DTO during pointer handling, and `window/presentation.rs` or paint
paths consume snapshots for regional repaint and hover overlays.

The split is declaration-only: field names, default values, helper method semantics, and public
re-export paths stay unchanged. The root data module still exports the same DTO names through
`data::HostMenuStateData`, `data::HostDragStateData`, `data::HostTextInputFocusData`,
`data::HostPaneInteractionStateData`, and `data::HostResizeStateData`.

## Validation

The 2026-06-21 host-interaction DTO split reduced `data/host_interaction.rs` from 109 lines to a
10-line structural entry. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p
zircon_editor --check`, a data host-interaction menu/drag/text-focus/pane/resize ownership scan,
scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full
Cargo tests remain deferred per the user's feature-first instruction.
