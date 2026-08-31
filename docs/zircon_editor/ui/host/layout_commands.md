---
related_code:
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/workbench/view/workbench_slot_to_view_host.rs
  - zircon_editor/src/ui/workbench/layout/manager/attach.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
implementation_files:
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
plan_sources:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - cargo test -p zircon_editor --lib window_host_manager --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-15: passed, 3 passed, 0 failed, 2020 filtered out)
  - cargo test -p zircon_editor --lib opening_functional_editor_window_creates_instance_scoped_floating_window --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-15: passed, 1 passed, 0 failed, 2022 filtered out)
doc_type: module-detail
---

# Editor Host Layout Commands

`EditorUiHost::open_view(...)` is the host boundary that turns a view descriptor into a concrete `ViewInstance` and attaches it to the current `WorkbenchLayout`.
For ordinary drawer and document views, the descriptor's preferred host can be used directly.

Functional editor windows need one extra normalization step. `PreferredHost::FloatingWindow` and `PreferredHost::ExclusiveMainPage` are descriptor-level defaults, so their placeholder page ids are expanded per opened instance:

- `FloatingWindow("floating")` becomes `FloatingWindow("window:{instance_id}")`;
- `ExclusivePage("exclusive")` becomes `ExclusivePage("page:{instance_id}")`.

When the resolved floating window does not yet exist, `attach_instance(...)` creates it with `LayoutCommand::DetachViewToWindow` and asks the native-window host manager to open the matching native window. The host manager allocates a per-window runtime `UiSurface` and exposes its tree id as `zircon.editor.native_window.window:{instance_id}`, so Window menu entries such as Material Editor and Animation Editor open as independent Unreal-style feature windows instead of failing on a missing placeholder window or sharing the main workbench surface.

Drawer-backed utility windows such as Asset Browser and Diagnostics now receive distinct exclusive page ids. They no longer collide on the shared `exclusive` placeholder when multiple utility windows are opened in one session.

## Global focus identity

Each document stack or floating window keeps its own active tab, while `EditorSessionState.focused_view` records the one view that owns editor-wide command and toolkit context. A `FocusView` command therefore reports a semantic change when that global identity changes even if the target was already active inside its local stack. This keeps command enablement and focused-toolkit menus correct across main-to-floating and floating-to-floating window switches.

Closing the globally focused view first selects the remaining active tab in the same drawer, document stack, or floating window. It falls back to the current main page only when the original host has disappeared or has no active tab. The session never retains a focus id that is absent from `open_view_instances`.
