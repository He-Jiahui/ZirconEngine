---
related_code:
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/focus.rs
  - zircon_editor/src/ui/workbench/layout/layout_command.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
implementation_files:
  - zircon_editor/src/ui/workbench/layout/manager/focus.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
doc_type: module-detail
---

# Workbench Layout Focus

`LayoutManager::focus_instance(...)` is the shared focus path for document tabs, drawer tabs, exclusive pages, and floating windows.
For the new JetBrains-style shell, focusing a view inside a collapsed drawer must reveal that drawer; otherwise normalization clears the active tab because collapsed drawers are not allowed to hold a selected view.

Drawer focus now mirrors explicit drawer tab activation:

- the focused drawer tab becomes `active_tab` and `active_view`;
- if the owning drawer is `Collapsed`, it is restored to `Pinned`;
- the active main page is switched to the page that owns the activity window;
- the legacy root drawer projection is resynced after the command through `apply.rs`.

This keeps command-driven focus, menu-driven view activation, and pointer-driven tab activation aligned for the default Material/Fyrox/JetBrains/Unreal workbench.
