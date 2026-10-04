---
related_code:
  - zircon_editor/src/ui/host/builtin_views/activity_views/activity_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/functional_panel_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/mod.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
implementation_files:
  - zircon_editor/src/ui/host/builtin_views/activity_views/activity_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/functional_panel_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/mod.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b (2026-05-11: passed)
  - cargo test -p zircon_editor --lib default_design_stack --locked --target-dir target/codex-shared-b (2026-05-11: passed, 2 passed)
  - cargo test -p zircon_editor --lib functional_editor_internal_view_descriptors_use_document_host --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
doc_type: module-detail
---

# Builtin Activity View Descriptors

Builtin activity-view descriptors define the registry-facing metadata for dockable, document, and drawer view contents. The new design stack creates stable view instances for functional editor internals, so descriptor coverage must include more than the legacy Workbench drawer views.

`functional_panel_view_descriptors.rs` adds descriptor coverage for functional editor page contents:

- `editor.prefab.viewport`;
- `editor.prefab.inspector`;
- `editor.material.graph`;
- `editor.material.preview`;
- `editor.ui.designer`;
- `editor.ui.source`;
- `editor.animation.timeline`;
- `editor.animation.graph`;
- `editor.asset_preview`;
- `editor.asset_metadata`.

These descriptors are separate from the `FyroxPanelPreset` component contracts. The panel preset says what logical component roles the panel needs; the builtin descriptor says how the host registry should title, constrain, gate, and place that view.

## Host Policy

Functional editor internals use `PreferredHost::DocumentCenter` and `DockPolicy::DocumentOnly` when they are primary page contents. Asset preview and metadata descriptors are drawer views, because they are secondary panes inside the drawer-backed Asset Browser window.

Capability filtering stays centralized in `with_builtin_required_capabilities(...)`. UI Designer and UI Source require the UI asset authoring subsystem. Animation Timeline and Animation Graph require animation authoring. The shared Workbench Scene/Game, Hierarchy, Inspector, Console, Asset Browser, Build Export, Runtime Diagnostics, and Plugin Manager descriptors keep their existing host policies.

## Alignment Test

`builtin_view_descriptors_cover_default_design_stack_view_instances` builds the default design stack, expands its `default_view_instances()`, and checks that every instance descriptor id is registered by `EditorManager`. This prevents adding a functional-window view to the preset without also exposing a registry descriptor for menu, layout, snapshot, and restore paths.
