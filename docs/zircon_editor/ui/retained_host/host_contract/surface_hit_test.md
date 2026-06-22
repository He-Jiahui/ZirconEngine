---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/action_id.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/option.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/viewport_toolbar_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/workbench.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/action_id.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/option.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/viewport_toolbar_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 implement editor UI architecture functionality first; delay full test matrix
tests:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; surface hit-test template-node subtree ownership scan; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after template-node hit-test subtree split: passed with existing warning noise only)
  - surface hit-test popup-row menu/option/hit/action ownership scan
  - surface hit-test viewport-toolbar production/test ownership scan
  - surface hit-test template-node surface-frame builder dispatch/node/surface ownership scan
  - surface hit-test template-node hit/model ownership scan
doc_type: module-detail
---

# Surface Hit Test Host Contract

The retained host surface hit-test module converts committed editor presentation geometry into pointer targets for native dispatch. `surface_hit_test/mod.rs` exposes the boundary entry points. `viewport_toolbar.rs` owns toolbar-control hit resolution. `surface_frame.rs` adapts runtime `UiSurfaceFrame` node hits into host-space pointer hits.

`viewport_toolbar_tests.rs` owns the viewport-toolbar regression fixture that builds a runtime `UiSurface`, attaches an interactive toolbar button, and proves the host hit payload preserves the shared surface-frame control geometry. Keeping that fixture outside `viewport_toolbar.rs` leaves the production owner focused on the hit DTO and shared surface-frame adapter call.

## Template Node Hit Testing

`surface_hit_test/template_node.rs` owns the public template-node pointer hit entry points:

- `hit_test_pane_template_node(...)` for pane body template nodes.
- `hit_test_workbench_window_template_node(...)` for componentized Workbench window nodes.
- `build_pane_template_surface_frame(...)` for constructing a hit-testable runtime `UiSurfaceFrame` from projected pane template nodes.

`template_node/model.rs` owns `TemplateNodePointerHit`, which carries control/action/binding ids, component role/family, text value, edit/commit action ids, and the final host-space hit frame consumed by native pointer dispatch. `template_node/hit.rs` owns ordinary surface-frame hit traversal after popup-row priority has had the first chance to consume or block a pointer hit.

## Subtree Ownership

`template_node/pane_nodes.rs` maps pane kinds to their projected `TemplatePaneNodeData` collections. This keeps the pane-kind registry separate from hit testing and surface-frame construction.

`template_node/popup_rows.rs` owns popup row priority for open template controls. It checks menu rows and option rows before ordinary surface-frame hits, while the `popup_rows/` children own the details: `menu.rs` synthesizes `workbench_menu_item` hits and blocks inside menu popup bodies, `option.rs` synthesizes `workbench_option` hits and blocks inside option popup bodies, `hit.rs` owns the popup-row hit DTO plus `TemplateNodePointerHit` assembly, and `action_id.rs` normalizes menu row action ids.

`template_node/surface_frame_builder.rs` is now a structural entry for temporary runtime `UiSurfaceFrame` construction. `surface_frame_builder/dispatch.rs` owns dispatchable-node filtering and component metadata classification, `surface_frame_builder/surface.rs` owns synthetic root creation, dispatchable-row traversal, runtime surface rebuild, and surface-frame return, and `surface_frame_builder/node.rs` owns per-template-node `UiTreeNode` assembly, input state flags, template metadata, and clip-frame preservation.

This split leaves `template_node.rs` as a 54-line entry module while keeping pointer-hit DTO storage, surface-frame hit traversal, popup-row behavior, and runtime surface-frame assembly in named children.

## Validation Notes

The 2026-06-21 popup-row menu/option/hit/action split reduced `template_node/popup_rows.rs` from 154 lines to a 35-line open-popup priority entry. `popup_rows/menu.rs` is 46 lines and owns structured menu-row hit testing, `popup_rows/option.rs` is 63 lines and owns structured option-row hit testing, `popup_rows/hit.rs` is 31 lines and owns `TemplatePopupRowHit` plus hit payload assembly, and `popup_rows/action_id.rs` is 30 lines and owns menu action-id normalization. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a surface hit-test popup-row menu/option/hit/action ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 viewport-toolbar production/test split reduced `surface_hit_test/viewport_toolbar.rs` from 125 lines to a 36-line production hit-test entry. `viewport_toolbar_tests.rs` now owns the two toolbar regressions and the runtime `UiSurface` fixture helper. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a viewport-toolbar production/test ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 template-node surface-frame builder split reduced `surface_hit_test/template_node/surface_frame_builder.rs` from 114 lines to a 7-line structural entry. `surface_frame_builder/dispatch.rs` owns dispatchability and component-role selection, `surface_frame_builder/surface.rs` owns runtime `UiSurface` root/traversal/rebuild, and `surface_frame_builder/node.rs` owns per-node `UiTreeNode` metadata, state flags, input policy, frame, and clip-frame assembly. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a template-node surface-frame builder dispatch/node/surface ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 template-node hit/model split reduced `surface_hit_test/template_node.rs` from 105 lines to a 54-line entry. `template_node/model.rs` owns `TemplateNodePointerHit`, while `template_node/hit.rs` owns popup-priority delegation, runtime surface-frame hit traversal, host-space frame translation, and pointer-hit payload assembly. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a template-node hit/model ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
