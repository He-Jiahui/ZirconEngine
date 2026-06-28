---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench\modules\generated\workbench_generated_bottom_drawer.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\generated\workbench_generated_bottom_panel.zui
  - zircon_editor/assets/ui/editor/host/generated_bottom_body.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_module_workspace.zui
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/generated_bottom_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/layout_drawers.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/generated_bottom.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_lifecycle.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/tests/host/retained_generated_bottom_template_body.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-generated-bottom-contract.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench\modules\generated\workbench_generated_bottom_drawer.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\generated\workbench_generated_bottom_panel.zui
  - zircon_editor/assets/ui/editor/host/generated_bottom_body.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_module_workspace.zui
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/generated_bottom_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/layout_drawers.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/generated_bottom.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_lifecycle.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/tests/host/retained_generated_bottom_template_body.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-generated-bottom-contract.mjs
plan_sources:
  - user: 2026-06-03 componentized editor UI prototype and native retained/Taffy replication request
  - docs/ui-and-layout/ai-workbench-style/component-prototype/README.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
tests:
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-generated-bottom-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
  - cargo test -p zircon_editor retained_generated_bottom_template_body --no-run
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_actions.rs zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_lifecycle.rs
doc_type: module-detail
---

# Generated Bottom Panel

The generated bottom panel records retained/Taffy drawer-host evidence plus shell bottom drawer pane body evidence for the browser prototype's shared `generatedBottomPanel()` secondary drawer generator. It does not claim full native-covered action/state consolidation yet: `workbench_generated_bottom_drawer.zui` remains hosted under `WorkbenchModuleWorkspace` for the current generated-bottom lifecycle path, while `generated_bottom_body.zui` registers a real `pane.generated_bottom.body` for the shell bottom drawer.

`workbench_generated_bottom_drawer.zui` owns the drawer host control id, the collapsed overlay state, and the `WorkbenchGeneratedBottomPanelHost` mount. `workbench_generated_bottom_panel.zui` declares visible shared panel content, mode tabs, filter controls, selected route detail rows, and one route row for every generated secondary bottom route produced by the native-synced core module registry. Those rows share the selector-safe `workbench-generated-bottom-route-row` class so the production `.zui` class governance can style them without CamelCase selector tokens. The shared panel is visible by default so `generated_bottom_body.zui` can host it directly in the shell bottom drawer; module-overlay show/hide remains owned by the drawer component and lifecycle helper. All events use the `WorkbenchGeneratedBottom/*` binding namespace so this evidence stays separate from core `WorkbenchModule/*` and More Editors `WorkbenchExtension/*` contracts. Preview action payloads use dotted functional paths such as `workbench.generated_bottom.gameplay_effect_compile_log.select`, so new generated-bottom actions stay sortable by function path instead of adding CamelCase action names.

`generated_bottom_body.zui` keeps `GeneratedBottomPanePanelHost` on an `Overlay` wrapper and mounts the same `WorkbenchGeneratedBottomPanel` as a child body, preserving the imported panel root control id for retained conversion. The body is registered through builtin template documents and component descriptors, then surfaced by `editor.generated_bottom` as a bottom-drawer `ActivityView` with `PanePayloadKind::GeneratedBottomV1`, `PaneRouteNamespace::Dock`, and template-only interaction. The builtin shell creates `editor.generated_bottom#1` in `ActivityDrawerSlot::Bottom`, and the bottom drawer layout appends it as a real shell tab after the build-export pane.

The pane projection path adds `ViewContentKind::GeneratedBottom`, `PanePayload::GeneratedBottomV1`, generated-bottom pane metadata, and a small generated-bottom payload builder. Retained presentation application converts generated-bottom panes into `GeneratedBottomPaneData`, and host-contract painter, hit-test, hover, and profiling paths accept `ViewContentKind::GeneratedBottom` template nodes so the shell body can reuse the generic retained template projection path.

`workbench_generated_bottom_template_bindings.rs` owns the template binding installation for the generated-bottom namespace. `generated_bottom_panel_actions.rs` owns retained action routing: it asks the lifecycle helper to open the drawer, switches generated-bottom mode tabs, selects route rows, opens generated-bottom dropdowns, and then hands off to feedback. `generated_bottom_panel_lifecycle.rs` owns the drawer visibility transitions and the drawer/panel control ids; opening exposes the host, drawer, and panel, while closing collapses the host and drawer without clearing retained panel state. `generated_bottom_panel_navigation.rs` owns the route target table, including the web `module-bottom-*:*` route, retained row control id, display module/panel labels, and selected mode tab. `generated_bottom_panel_feedback.rs` consumes that route target to update the shared selected route, module, panel, mode, and status controls.

`componentized_window.rs` only detects the generated-bottom action namespace and delegates to `generated_bottom_panel_actions.rs`; when switching to ordinary module or extension workspaces, it delegates drawer closing to `generated_bottom_panel_lifecycle.rs`. This keeps the retained generated-bottom surface behind an explicit drawer boundary rather than leaving the panel as another core module body or another large branch inside the window bridge.

The contract verifier derives the web route list from `nativeModules`, so a new generated secondary bottom tab cannot silently miss native evidence. Passing `verify-native-generated-bottom-contract.mjs` means retained drawer host, visible shell bottom drawer pane body, content, action routing, bindings, preview actions, host-contract projection, and route feedback exist for the generator family. Full promotion still requires unifying shell-pane action/state ownership with the existing module lifecycle contract.
