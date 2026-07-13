---
related_code:
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/core/commands/menu_model.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/tests/editor_event/runtime/mod.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
implementation_files:
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/core/commands/menu_model.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
  - .codex/plans/Zircon Editor Demo 首屏与 .zui 组件陈列计划.md
tests:
  - zircon_editor/src/tests/editor_event/runtime/mod.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b (2026-05-11: passed)
  - cargo test -p zircon_editor --lib workbench_window_menu_exposes_unreal_style_functional_windows --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib editor_operation_registry_exposes_builtin_menu_operations_by_path --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib workbench_view_model_projects_menu_strip_drawers_and_status --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib workbench_window_menu_exposes_unreal_style_functional_windows --locked --target-dir target/codex-shared-b (2026-05-15: passed, 1 passed)
  - cargo test -p zircon_editor --lib editor_operation_registry_exposes_builtin_menu_operations_by_path --locked --target-dir target/codex-shared-b (2026-05-15: passed, 1 passed)
doc_type: module-detail
---

# Workbench Window Menu

The Workbench `Window` menu is the command-level entry point for Unreal-style functional editor windows. The menu still uses the existing `MenuAction::OpenView(...)` event, but the descriptor ids now target the preset-aligned activity-window descriptors rather than legacy view-only entries.

The functional window entries are:

- `Prefab Editor` -> `editor.prefab_editor_window` -> `window.prefab_editor.open`;
- `Material Editor` -> `editor.material_editor_window` -> `window.material_editor.open`;
- `UI Component Showcase` -> `editor.ui_component_showcase` -> `window.ui_component_showcase.open`;
- `Material Demo` -> `editor.material_demo_window` -> `window.material_demo.open`;
- `UI Asset Editor` -> `editor.ui_asset_editor_window` -> `window.ui_asset_editor.open`;
- `Animation Editor` -> `editor.animation_editor_window` -> `window.animation_editor.open`;
- `Asset Browser` -> `editor.asset_browser_window` -> `window.asset_browser.open`;
- `Diagnostics` -> `editor.diagnostics_window` -> `window.diagnostics.open`.

`UI Component Showcase` is intentionally in `Window` because it is now the no-argument editor front screen and uses an exclusive activity-window page. `Material Demo` remains available as a separate diagnostic/demo entry but is no longer the Welcome startup demo route. `Debug Observatory` and `Reset Layout` remain in the same menu. The View menu keeps existing drawer/document view entries for compatibility, but new feature editors should be exposed through `Window` when they represent a top-level editing unit.

`operation_path_for_menu_action(...)` maps descriptor ids to stable operation paths, while the context-owned `EditorCommandRegistry` is the single source for the corresponding `Window/...` command metadata. Menu projection, native binding payloads, CLI/remote lookup, and journal operation ids therefore share one typed id space and one runtime registry handle.
