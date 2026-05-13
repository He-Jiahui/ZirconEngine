---
related_code:
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/ui/workbench/model/menu/window_menu.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
implementation_files:
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/ui/workbench/model/menu/window_menu.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b (2026-05-11: passed)
  - cargo test -p zircon_editor --lib workbench_window_menu_exposes_unreal_style_functional_windows --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib editor_operation_registry_exposes_builtin_menu_operations_by_path --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib workbench_view_model_projects_menu_strip_drawers_and_status --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
doc_type: module-detail
---

# Workbench Window Menu

The Workbench `Window` menu is the command-level entry point for Unreal-style functional editor windows. The menu still uses the existing `MenuAction::OpenView(...)` event, but the descriptor ids now target the preset-aligned activity-window descriptors rather than legacy view-only entries.

The functional window entries are:

- `Prefab Editor` -> `editor.prefab_editor_window` -> `Window.PrefabEditor.Open`;
- `Material Editor` -> `editor.material_editor_window` -> `Window.MaterialEditor.Open`;
- `UI Asset Editor` -> `editor.ui_asset_editor_window` -> `Window.UiAssetEditor.Open`;
- `Animation Editor` -> `editor.animation_editor_window` -> `Window.AnimationEditor.Open`;
- `Asset Browser` -> `editor.asset_browser_window` -> `Window.AssetBrowser.Open`;
- `Diagnostics` -> `editor.diagnostics_window` -> `Window.Diagnostics.Open`.

`Debug Observatory` and `Reset Layout` remain in the same menu. The View menu keeps existing drawer/document view entries for compatibility, but new feature editors should be exposed through `Window` when they represent a top-level editing unit.

`operation_path_for_menu_action(...)` maps the new descriptor ids to stable operation paths, and `EditorOperationRegistry::with_builtin_operations()` registers the same paths with `Window/...` menu paths. This keeps menu projection, native binding payloads, remote operation lookup, and journal operation ids aligned.
