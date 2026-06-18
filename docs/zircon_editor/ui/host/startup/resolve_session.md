---
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_editor/src/ui/host/startup/persistence.rs
  - zircon_editor/src/ui/host/startup/validation.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document.rs
  - zircon_editor/src/ui/workbench/startup/stored_startup_session.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
implementation_files:
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_editor/src/ui/host/startup/persistence.rs
  - zircon_editor/src/ui/host/startup/validation.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
plan_sources:
  - user: 2026-05-15 continue Zircon Editor Demo front screen plan
  - .codex/plans/Zircon Editor Demo 首屏与 .zui 组件陈列计划.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - cargo test -p zircon_editor --lib startup_session_defaults_to_component_showcase_without_recent_project --locked --target-dir target/codex-shared-b (2026-05-15: passed)
  - cargo test -p zircon_editor --lib create_project_and_open_persists_recent_project_and_returns_project_session --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15: passed)
  - cargo test -p zircon_editor --lib explicit_project_open_session_bypasses_component_showcase_builtin_view --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15: passed)
  - cargo test -p zircon_editor --lib welcome_startup_demo_routes_to_component_showcase_window --locked --target-dir target/codex-shared-b (2026-05-15: passed)
  - cargo test -p zircon_editor --lib component_showcase_window_descriptor_opens_as_exclusive_demo_page --locked --target-dir target/codex-shared-b (2026-05-15: passed)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/host/startup/resolve_session.rs zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs (2026-06-16: passed)
  - cargo test -p zircon_editor --lib startup_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed, 2 passed, 0 failed, 2024 filtered out)
  - cargo test -p zircon_editor --lib create_project_and_open_persists_recent_project_and_returns_project_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed, 1 passed, 0 failed, 2025 filtered out)
  - cargo test -p zircon_editor --lib explicit_project_open_session_bypasses_component_showcase_builtin_view --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed, 1 passed, 0 failed, 2025 filtered out)
doc_type: module-detail
---

# Editor Startup Session Resolution

## Purpose

The no-argument editor startup path resolves the stored editor session before choosing a visible front screen. `EditorUiHost::resolve_startup_session()` loads `StoredStartupSession`, validates the last remembered project, and restores it when it is still a valid project. The returned `EditorStartupSessionDocument` then uses `mode = Project`, carries the loaded `EditorProjectDocument`, keeps `open_builtin_view = None`, and lets startup state construction apply the project workspace before creating the runtime level.

When there is no remembered project, or the remembered project is missing, invalid, or fails to open, startup falls back to the UI Component Showcase by returning `open_builtin_view = "editor.ui_component_showcase"`. The fallback session still includes the validated recent-project list and a status line explaining why automatic restore was skipped.

Explicit project startup still bypasses this path. `EditorGuiStartupRequestArgs` maps `--project` to `EditorGuiStartupRequest::OpenProject` and `--create-project ... --template renderable-empty` to `CreateProject`; `resolve_editor_startup_session()` then calls `open_project_and_remember(...)` or `create_project_and_open(...)`, which return normal `Project` sessions with `open_builtin_view = None`.

## Behavior Model

Project restore uses `EditorUiHost::open_project(...)` only after validation says the stored `last_project_path` is usable. That keeps the automatic path inside the editor host boundary: the runtime asset manager is opened, the editor asset manager refreshes from the runtime project, the UI asset watcher restarts, and the editor project document loads the runtime scene plus recoverable project workspace metadata.

`EditorStartupSessionDocument.open_builtin_view` is a post-resolution instruction, not a serialized project document. `build_startup_state(...)` sees the field on fallback sessions, dismisses the Welcome page if it exists, opens the descriptor through `EditorManager::open_view(...)`, and then sets the runtime session mode to `Project` so the Welcome page does not remain the active shell. It intentionally leaves `project_open = false`; the component showcase is a built-in editor view, not a remembered project.

The UI Component Showcase descriptor uses `PreferredHost::ExclusiveMainPage`, so startup opens `page:editor.ui_component_showcase#1` instead of adding the demo behind Scene/Game tabs in the default Workbench document center. The Welcome `OpenStartupDemo` button routes to the same descriptor and status text, while the Workbench button remains the explicit route back to the normal Workbench shell.

## Edge Cases

If the last project is invalid, the fallback message uses the validation state: missing project, invalid manifest, or invalid project. If validation succeeds but opening fails, the same fallback keeps the open error in the status message and opens the component showcase. `recent_projects` is built with validation for the fallback path so the Welcome surface can mark stale entries without discarding them.

If a restored project has a missing or unreadable `.zircon/editor-workspace.json`, `EditorProjectDocument` records a workspace restore diagnostic but still opens the project. The startup status becomes `Restored recent project with default layout`, and later `apply_project_workspace(None)` falls through the global-default and builtin-layout chain.

If a caller builds a project session and also sets `open_builtin_view`, the built-in view request wins because it represents a front-screen routing decision.

## Test Coverage

Focused tests on 2026-06-16 cover the no-argument component-showcase default with no stored project, automatic restore after create/open persists a valid recent project, missing-last-project fallback to the component showcase with `RecentProjectValidation::Missing`, and explicit project-open startup bypassing `open_builtin_view`. Earlier 2026-05-15 evidence still covers Welcome Demo routing and exclusive-page descriptor host.
