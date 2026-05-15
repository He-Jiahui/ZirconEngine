---
related_code:
  - zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/mod.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/functional_window.rs
  - zircon_editor/src/ui/workbench/model/menu/window_menu.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
implementation_files:
  - zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/mod.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/model/menu/window_menu.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/core/editor_operation.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
  - .codex/plans/Zircon Editor Demo 首屏与 .zui 组件陈列计划.md
tests:
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b (2026-05-11: passed)
  - cargo test -p zircon_editor --lib builtin_activity_windows --locked --target-dir target/codex-shared-b (2026-05-11: passed, 2 passed after rerun with longer timeout)
  - cargo test -p zircon_editor --lib unreal_style_feature_window_descriptors_use_expected_hosts --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib workbench_window_menu_exposes_unreal_style_functional_windows --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib editor_operation_registry_exposes_builtin_menu_operations_by_path --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_window_descriptor_opens_as_exclusive_demo_page --locked --target-dir target/codex-shared-b (2026-05-15: passed, 1 passed)
doc_type: module-detail
---

# Builtin Activity Window Descriptors

Builtin activity-window descriptors are the editor host's registry-facing definitions for windows that can be opened, restored, or projected from layout data. The Material/Fyrox/JetBrains/Unreal preset now declares Unreal-style functional windows in `EditorUiDesignStack`, so the builtin descriptor set must cover those generated `ActivityWindowLayout.descriptor_id` values.

`functional_window_view_descriptors.rs` adds descriptor coverage for:

- `editor.ui_component_showcase`;
- `editor.scene_game_window`;
- `editor.prefab_editor_window`;
- `editor.material_editor_window`;
- `editor.ui_asset_editor_window`;
- `editor.animation_editor_window`;
- `editor.asset_browser_window`;
- `editor.diagnostics_window`.

The old descriptors such as `editor.prefab`, `editor.asset_browser`, and `editor.ui_asset` remain registered for existing menu and asset-editor paths. The new `_window` descriptors are the preset-aligned contract used by the functional-window model. `editor.ui_component_showcase` is not a `_window` id, but it now participates in the same activity-window registry because it is the editor's demo front screen.

`UnrealWindowModelPreset` is the source model for those functional windows. The builtin descriptor layer is the host-facing registry coverage for that model: every window descriptor id generated from the preset must be present in `EditorManager::descriptors()` before the layout can open or restore that window.

The Workbench `Window` menu now opens these preset-aligned descriptors directly. It maps Prefab, Material, UI Asset, Animation, Asset Browser, and Diagnostics entries to `Window.*.Open` operation paths, so the menu model, operation registry, and activity-window descriptor IDs use the same functional-window vocabulary.

## Host Policy

Floating feature editors, including Prefab, Material, UI Asset, and Animation, use `PreferredHost::FloatingWindow` and are multi-instance. Drawer-backed utility windows such as Asset Browser and Diagnostics use `PreferredHost::ExclusiveMainPage`. The UI Component Showcase also uses `PreferredHost::ExclusiveMainPage` so no-argument startup opens the demo as `page:editor.ui_component_showcase#1` instead of adding it behind Scene/Game tabs in the Workbench document center. The Workbench descriptor remains the embedded main frame source.

`EditorUiHost::open_view(...)` expands those descriptor-level host preferences into concrete instance-scoped targets. Floating feature editors open in `window:{instance_id}` native floating windows, while drawer-backed utility windows open in `page:{instance_id}` exclusive pages. This keeps the generic descriptor defaults reusable while preventing multiple functional windows from colliding on the same placeholder target.

Capability filtering stays in `with_builtin_required_capabilities(...)`. Animation and UI Asset windows keep their subsystem gates. Runtime Diagnostics keeps the diagnostics gate. Native-window requirements are applied to windows that need the native hosting path for the current preset.

## Alignment Test

`builtin_activity_windows_cover_default_design_stack_windows` projects the default design stack into a `WorkbenchLayout` and verifies that every activity-window descriptor id is registered by `EditorManager`. This prevents future preset changes from introducing layout windows that the view registry cannot recognize.
