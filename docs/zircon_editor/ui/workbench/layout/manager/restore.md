---
related_code:
  - zircon_editor/src/ui/host/builtin_layout/hybrid_layout.rs
  - zircon_editor/src/ui/workbench/layout/manager/defaults.rs
  - zircon_editor/src/ui/workbench/layout/manager/restore.rs
  - zircon_editor/src/ui/workbench/layout/restore_policy.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
implementation_files:
  - zircon_editor/src/ui/workbench/layout/manager/restore.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/ui/workbench/layout/manager/restore.rs
doc_type: module-detail
---

# Workbench Layout Restore

`LayoutManager::restore_workspace(...)` resolves persisted editor layout state according to `RestorePolicy`.
The restore order is unchanged: project workspace, global default, and optional explicit preset keep their policy-defined priority.

When every persisted source is missing, restore now falls back through `LayoutManager::default_layout()` instead of `WorkbenchLayout::default()`.
That keeps first-run restore aligned with the Material/Fyrox/JetBrains/Unreal preset that powers `builtin_hybrid_layout()`:

- Scene and Game reopen as central document tabs.
- Hierarchy and Asset Browser return to the left Fyrox-style drawer.
- Inspector, Console, Diagnostics, Build Export, and Plugin Manager keep their preset drawer placement.
- Functional editor windows such as Material Editor and Animation Editor remain registered as activity windows.

This matters because reset, startup, and restore now share the same preset-derived default. The legacy empty workbench model still exists as a low-level data fallback, but it is no longer the product default for editor restoration.
