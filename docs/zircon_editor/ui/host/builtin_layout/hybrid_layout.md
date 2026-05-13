---
related_code:
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/ensure_shell_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/hybrid_layout.rs
  - zircon_editor/src/ui/host/builtin_layout/mod.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
implementation_files:
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/hybrid_layout.rs
  - zircon_editor/src/ui/host/builtin_layout/mod.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed)
  - cargo test -p zircon_editor --lib default_layout --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 5 passed)
  - cargo test -p zircon_editor --lib applying_project_workspace_preserves_builtin_shell_drawers --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib bootstrap_and_startup --locked --target-dir target/codex-shared-b (2026-05-11: passed, 10 passed)
doc_type: module-detail
---

# Builtin Hybrid Layout

`builtin_hybrid_layout_for_subsystems(...)` is the editor startup bridge that turns the current product UI preset into the first `WorkbenchLayout` used by `EditorManager` bootstrap. It deliberately keeps the old host-facing API name because callers still ask for a builtin hybrid layout, but the source of truth is now the componentized Material/Fyrox/JetBrains/Unreal workbench preset.

The startup path constructs `EditorUiDesignStack::material_fyrox_jetbrains_unreal()` and projects it through `default_workbench_layout()`. That gives startup the same window, document, drawer, and functional-window model used by the preset tests:

- central `editor.scene#1` and `editor.game#1` document tabs;
- left drawer `editor.hierarchy#1` and `editor.assets#1`, with Hierarchy selected;
- right drawer `editor.inspector#1`;
- bottom drawer `editor.console#1`, `editor.runtime_diagnostics#1`, and `editor.build_export_desktop#1`;
- lower-left drawer `editor.module_plugins#1`;
- registered activity windows for Workbench, Scene/Game, Prefab Editor, Material Editor, UI Asset Editor, Animation Editor, Asset Browser, and Diagnostics.

Workbench drawer slot assignment and startup drawer mode come from `JetBrainsShellPreset`, not from the older hand-built startup layout. The lower-left Module Plugins drawer keeps its tab but starts collapsed because the Modules shell drawer has `default_mode = Collapsed`.

## Capability Filtering

The only editor-subsystem adjustment made in `hybrid_layout.rs` is runtime diagnostics filtering. When `EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS` is disabled, `editor.runtime_diagnostics#1` is removed from both the legacy root drawer map and every activity-window drawer map. If that removal empties a drawer, the drawer is collapsed and its active tab/view is cleared. If the removed view was active but siblings remain, the first remaining tab becomes active.

This keeps the startup layout deterministic while preserving the subsystem gate that existed before the preset cutover.

## Shell Instance Seed

`builtin_shell_view_instances(...)` now seeds the default shell with the views required by the preset startup layout instead of the legacy Project-first drawer set. `editor.project#1` is no longer included in the default shell instances. `editor.assets#1` remains a left drawer view, but its visible title is `Asset Browser` to match the Fyrox-style panel role.

The seed still includes the existing view descriptors for scene/game, hierarchy, inspector, console, runtime diagnostics, build/export, and module plugins. `ensure_builtin_shell_instances(...)` continues to repair missing builtin instances when project workspaces are applied, so a project-specific workspace should preserve the new default drawer baseline rather than reviving the old Project drawer.

## Legacy Sources

The previous hand-built startup sources `layout_drawers.rs` and `workbench_page.rs` remain in the repository as legacy references, but `builtin_layout::mod` no longer compiles them into the startup module graph. New workbench behavior should be routed through `zircon_editor::ui::workbench::preset` first, then adapted in this builtin bridge only when startup needs a capability-specific filter.
