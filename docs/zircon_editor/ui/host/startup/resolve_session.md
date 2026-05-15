---
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
implementation_files:
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
plan_sources:
  - user: 2026-05-15 continue Zircon Editor Demo front screen plan
  - .codex/plans/Zircon Editor Demo 首屏与 .zui 组件陈列计划.md
tests:
  - cargo test -p zircon_editor --lib startup_session_defaults_to_component_showcase_without_recent_project --locked --target-dir target/codex-shared-b (2026-05-15: passed)
  - cargo test -p zircon_editor --lib create_project_and_open_persists_recent_project_and_returns_project_session --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15: passed)
  - cargo test -p zircon_editor --lib explicit_project_open_session_bypasses_component_showcase_builtin_view --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15: passed)
  - cargo test -p zircon_editor --lib welcome_startup_demo_routes_to_component_showcase_window --locked --target-dir target/codex-shared-b (2026-05-15: passed)
  - cargo test -p zircon_editor --lib component_showcase_window_descriptor_opens_as_exclusive_demo_page --locked --target-dir target/codex-shared-b (2026-05-15: passed)
doc_type: module-detail
---

# Editor Startup Demo Session

## Purpose

The no-argument editor startup path is now a demo-first route. `EditorUiHost::resolve_startup_session()` returns an `EditorStartupSessionDocument` with `open_builtin_view = "editor.ui_component_showcase"` and does not reopen the stored last project or validate the recent-project list. This makes `zircon_editor.exe` land on the UI Component Showcase unless the user explicitly supplies a project-oriented CLI request.

Explicit project startup still bypasses this path. `EditorGuiStartupRequestArgs` maps `--project` to `EditorGuiStartupRequest::OpenProject` and `--create-project ... --template renderable-empty` to `CreateProject`; `resolve_editor_startup_session()` then calls `open_project_and_remember(...)` or `create_project_and_open(...)`, which return normal `Project` sessions with `open_builtin_view = None`.

## Behavior Model

`EditorStartupSessionDocument.open_builtin_view` is a post-resolution instruction, not a serialized project document. `build_startup_state(...)` sees the field, dismisses the Welcome page if it exists, opens the descriptor through `EditorManager::open_view(...)`, and then sets the runtime session mode to `Project` so the Welcome page does not remain the active shell. It intentionally leaves `project_open = false`; the component showcase is a built-in editor view, not a remembered project.

The UI Component Showcase descriptor uses `PreferredHost::ExclusiveMainPage`, so startup opens `page:editor.ui_component_showcase#1` instead of adding the demo behind Scene/Game tabs in the default Workbench document center. The Welcome `OpenStartupDemo` button routes to the same descriptor and status text, while the Workbench button remains the explicit route back to the normal Workbench shell.

## Edge Cases

The stored startup session is still used for recent-project commands and explicit project open/create flows, but no-argument startup deliberately ignores `last_project_path`. That prevents a stale or heavy project reopen from hiding the component demo. If a caller builds a project session and also sets `open_builtin_view`, the built-in view request wins because it represents a front-screen routing decision.

## Test Coverage

Focused tests on 2026-05-15 covered the no-argument default, explicit create/open project behavior, explicit project-open startup bypassing `open_builtin_view`, Welcome Demo routing, and exclusive-page descriptor host. The current plan evidence also includes `cargo check -p zircon_editor --lib --locked`.
