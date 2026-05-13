---
related_code:
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/workbench/view/preferred_host_to_view_host.rs
  - zircon_editor/src/ui/workbench/layout/manager/attach.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
implementation_files:
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
doc_type: module-detail
---

# Editor Host Layout Commands

`EditorUiHost::open_view(...)` is the host boundary that turns a view descriptor into a concrete `ViewInstance` and attaches it to the current `WorkbenchLayout`.
For ordinary drawer and document views, the descriptor's preferred host can be used directly.

Functional editor windows need one extra normalization step. `PreferredHost::FloatingWindow` and `PreferredHost::ExclusiveMainPage` are descriptor-level defaults, so their placeholder page ids are expanded per opened instance:

- `FloatingWindow("floating")` becomes `FloatingWindow("window:{instance_id}")`;
- `ExclusivePage("exclusive")` becomes `ExclusivePage("page:{instance_id}")`.

When the resolved floating window does not yet exist, `attach_instance(...)` creates it with `LayoutCommand::DetachViewToWindow` and asks the native-window host manager to open the matching native window. This makes Window menu entries such as Material Editor and Animation Editor open as independent Unreal-style feature windows instead of failing on a missing placeholder window.

Drawer-backed utility windows such as Asset Browser and Diagnostics now receive distinct exclusive page ids. They no longer collide on the shared `exclusive` placeholder when multiple utility windows are opened in one session.
