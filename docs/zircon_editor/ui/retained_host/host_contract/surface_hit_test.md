---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/route_hit.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/workbench.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/route_hit.rs
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
plan_sources:
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 implement editor UI architecture functionality first; delay full test matrix
tests:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; surface hit-test template-node subtree ownership scan; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after template-node hit-test subtree split: passed with existing warning noise only)
  - surface hit-test popup-row menu/option/hit/action ownership scan
  - surface hit-test template-node surface-frame builder dispatch/node/surface ownership scan
  - surface hit-test template-node hit/model ownership scan
  - tools/tests/test_editor_pane_route_borrowed_hit_performance_contract.py
  - M5.S4 source scan confirming no viewport toolbar surface-hit-test owner remains
  - cargo test -p zircon_editor --lib retained_viewport_toolbar_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-focused-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s4-integration-0623 --message-format short --color never -- --test-threads=1
doc_type: module-detail
---

# Surface Hit Test Host Contract

The retained host surface hit-test module converts committed editor presentation geometry into pointer targets for native template-node dispatch. `surface_hit_test/mod.rs` exposes the template-node boundary entry points. `surface_frame.rs` adapts runtime `UiSurfaceFrame` node hits into host-space pointer hits for projected pane and workbench template nodes.

Viewport toolbar control dispatch no longer lives in this module. M5.S4 moved it to the route-intent path: native toolbar routing consults the submitted `PaneData.viewport.toolbar_surface_frame` only to preserve the clicked control id for damage, while the callback dispatch syncs the same projected frame into `ViewportToolbarPointerBridge` and resolves commands through route ids.

## Template Node Hit Testing

`surface_hit_test/template_node.rs` owns the public template-node pointer hit entry points:

- `hit_test_pane_template_node(...)` for pane body template nodes.
- `hit_test_pane_template_node_borrowed(...)` for generation-scoped pane routes that must not allocate an owned callback payload.
- `hit_test_workbench_window_template_node(...)` for componentized Workbench window nodes.
- `build_pane_template_surface_frame(...)` for constructing a hit-testable runtime `UiSurfaceFrame` from projected pane template nodes.

`template_node/model.rs` owns the callback-facing `TemplateNodePointerHit` plus borrowed `TemplateNodePointerRouteHit` and move-only views. `template_node/hit.rs` owns the single ordinary/popup geometry selection path and returns generation-borrowed pane hits; `template_node/route_hit.rs` is the only pane activation conversion into the owned callback payload.

## Subtree Ownership

`template_node/pane_nodes.rs` maps pane kinds to their projected `TemplatePaneNodeData` collections. This keeps the pane-kind registry separate from hit testing and surface-frame construction.

`template_node/popup_rows.rs` owns popup row priority for open template controls. It checks menu rows and option rows before ordinary surface-frame hits, while the `popup_rows/` children own the details: `menu.rs` and `option.rs` return one borrowed popup target or a blocking result, `hit.rs` projects that target into borrowed route/move data or the existing owned Workbench payload, and `action_id.rs` normalizes menu row action ids only at an owned activation boundary.

`template_node/surface_frame_builder.rs` is now a structural entry for temporary runtime `UiSurfaceFrame` construction. `surface_frame_builder/dispatch.rs` owns dispatchable-node filtering and component metadata classification, `surface_frame_builder/surface.rs` owns synthetic root creation, dispatchable-row traversal, runtime surface rebuild, and surface-frame return, and `surface_frame_builder/node.rs` owns per-template-node `UiTreeNode` assembly, input state flags, template metadata, and clip-frame preservation.

This split leaves `template_node.rs` as a 54-line entry module while keeping pointer-hit DTO storage, surface-frame hit traversal, popup-row behavior, and runtime surface-frame assembly in named children.

## Validation Notes

The 2026-06-21 popup-row menu/option/hit/action split reduced `template_node/popup_rows.rs` from 154 lines to a 35-line open-popup priority entry. `popup_rows/menu.rs` is 46 lines and owns structured menu-row hit testing, `popup_rows/option.rs` is 63 lines and owns structured option-row hit testing, `popup_rows/hit.rs` is 31 lines and owns `TemplatePopupRowHit` plus hit payload assembly, and `popup_rows/action_id.rs` is 30 lines and owns menu action-id normalization. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a surface hit-test popup-row menu/option/hit/action ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-23 M5.S4 cleanup deleted `surface_hit_test/viewport_toolbar.rs` and `surface_hit_test/viewport_toolbar_tests.rs`. The old toolbar hit payload is replaced by `route_intent` and `ViewportToolbarPointerBridge::sync_surface_frame(...)`, leaving this module scoped to template-node hit testing. Validation used direct rustfmt over touched retained-host files, a source scan confirming no `ViewportToolbarPointerHit`, `hit_test_viewport_toolbar`, old active-control cache, or old `PanePointerTarget::ViewportToolbar(...)` shape remains, the focused `retained_viewport_toolbar_pointer` suite passing 7/7, and the integration contracts passing 27/27 after updating stale contract paths to current `.zui` and folder-backed owner locations. The editor host app compile smoke also passes offline; full interactive window regression remains a manual follow-up. `Cargo.lock` was restored to the protected hash after each no-locked/offline Cargo validation.

The 2026-06-21 template-node surface-frame builder split reduced `surface_hit_test/template_node/surface_frame_builder.rs` from 114 lines to a 7-line structural entry. `surface_frame_builder/dispatch.rs` owns dispatchability and component-role selection, `surface_frame_builder/surface.rs` owns runtime `UiSurface` root/traversal/rebuild, and `surface_frame_builder/node.rs` owns per-node `UiTreeNode` metadata, state flags, input policy, frame, and clip-frame assembly. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a template-node surface-frame builder dispatch/node/surface ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 template-node hit/model split reduced `surface_hit_test/template_node.rs` from 105 lines to a 54-line entry. `template_node/model.rs` owns `TemplateNodePointerHit`, while `template_node/hit.rs` owns popup-priority delegation, runtime surface-frame hit traversal, host-space frame translation, and pointer-hit payload assembly. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a template-node hit/model ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
