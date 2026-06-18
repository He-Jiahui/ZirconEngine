---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/workbench.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/viewport_toolbar.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 implement editor UI architecture functionality first; delay full test matrix
tests:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; surface hit-test template-node subtree ownership scan; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after template-node hit-test subtree split: passed with existing warning noise only)
doc_type: module-detail
---

# Surface Hit Test Host Contract

The retained host surface hit-test module converts committed editor presentation geometry into pointer targets for native dispatch. `surface_hit_test/mod.rs` exposes the boundary entry points. `viewport_toolbar.rs` owns toolbar-control hit resolution. `surface_frame.rs` adapts runtime `UiSurfaceFrame` node hits into host-space pointer hits.

## Template Node Hit Testing

`surface_hit_test/template_node.rs` owns the public template-node pointer hit contract and entry points:

- `hit_test_pane_template_node(...)` for pane body template nodes.
- `hit_test_workbench_window_template_node(...)` for componentized Workbench window nodes.
- `build_pane_template_surface_frame(...)` for constructing a hit-testable runtime `UiSurfaceFrame` from projected pane template nodes.

The root module also owns `TemplateNodePointerHit`, which carries control/action/binding ids, component role/family, text value, edit/commit action ids, and the final host-space hit frame consumed by native pointer dispatch.

## Subtree Ownership

`template_node/pane_nodes.rs` maps pane kinds to their projected `TemplatePaneNodeData` collections. This keeps the pane-kind registry separate from hit testing and surface-frame construction.

`template_node/popup_rows.rs` owns popup row priority for open template controls. It checks menu rows and option rows before ordinary surface-frame hits, synthesizes `workbench_menu_item` and `workbench_option` hits, blocks clicks inside open popup bodies that do not hit an enabled row, and normalizes menu row action ids.

`template_node/surface_frame_builder.rs` owns dispatchable-node filtering and construction of the temporary runtime `UiSurfaceFrame` used for normal template-node hit testing. It inserts a synthetic root, skips non-dispatchable or disabled nodes, attaches template metadata, preserves clip frames, rebuilds the runtime surface, and returns its surface frame.

This split leaves `template_node.rs` as a 105-line entry and DTO module while keeping popup-row behavior and runtime surface-frame assembly in named children.
