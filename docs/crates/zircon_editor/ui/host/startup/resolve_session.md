---
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_editor/src/ui/host/editor_manager_startup.rs
  - zircon_editor/src/core/project/authority/mod.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/assets/ui/editor/welcome.zui
  - zircon_editor/assets/ui/editor/component_showcase.zui
implementation_files:
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_editor/src/ui/host/editor_manager_startup.rs
  - zircon_editor/src/core/project/authority/mod.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/assets/ui/editor/welcome.zui
plan_sources:
  - user: 2026-05-15 continue Zircon Editor Demo front screen plan
  - .codex/plans/Zircon Editor Demo 首屏与 .zui 组件陈列计划.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup/session_startup.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
doc_type: module-detail
---

# Editor Startup Session Resolution

## Purpose

The no-argument editor startup path builds a Welcome project chooser. `EditorUiHost::resolve_startup_session()` reads the validated recent-project projection but does not select or open one. The returned `EditorStartupSessionDocument` uses `mode = Welcome`, has neither a project document nor `open_builtin_view`, and leaves project execution for an explicit launch action.

Valid, migration-required, stale, and invalid recent entries remain available to the chooser as a non-authoritative projection. They do not constitute permission to restore a project. A legacy manifest is marked `RequiresMigration` rather than `Valid`, so it cannot be mistaken for an immediately launchable project before Copy, Convert In-place, or Cancel is chosen. This prevents a no-request startup from loading runtime, scripts, plugins, native extensions, or recovery state before the ProjectLaunchIntent, preflight, and admission chain has made an explicit decision.

Explicit project startup still bypasses this path. `EditorGuiStartupRequestArgs` converts direct
CLI flags or deserializes a versioned `--project-launch-intent` into the one
`EditorGuiStartupRequest::Project { intent }` shape. A Hub handshake accepts only a
Hub-originated transport intent, preserving its operation identity across the process boundary.
The host executes that intent through data-only preflight before it may open a normal `Project`
session with `open_builtin_view = None`.

## Behavior Model

No-request startup only reads recent-project metadata and produces the chooser state. Opening a
project stays on the explicit project intent route and begins with data-only preflight; the chooser
does not provide a second automatic activation route. Session admission, activation, and first
present remain later lifecycle gates and are not implied by an accepted launch intent.

`EditorStartupSessionDocument.open_builtin_view` remains a post-resolution instruction, not a serialized project document. It is used only by explicit built-in view requests, such as the Welcome demo action; no-argument startup leaves it unset and keeps the Welcome page active.

The UI Component Showcase descriptor uses `PreferredHost::ExclusiveMainPage`, so startup opens `page:editor.ui_component_showcase#1` instead of adding the demo behind Scene/Game tabs in the default Workbench document center. The Welcome `OpenStartupDemo` button routes to the same descriptor and status text, while the Workbench button remains the explicit route back to the normal Workbench shell.

## Edge Cases

If a recent project is missing, invalid, or otherwise stale, its validation remains visible in the chooser without trying to open it. A valid entry also remains inert until a user action produces an explicit launch request.

If an explicitly opened project has a missing or unreadable `.zircon/editor-workspace.json`, `EditorProjectDocument` records a workspace restore diagnostic but still opens the project. `apply_project_workspace(None)` then falls through the global-default and builtin-layout chain.

If a caller builds a project session and also sets `open_builtin_view`, the built-in view request wins because it represents a front-screen routing decision.

## Test Coverage

Focused tests cover no-argument chooser startup with and without valid recent entries, normalization of a remembered manifest path without activation, explicit project open, and the source boundary that rejects the removed callback auto-open path. Separate retained-host coverage owns explicit Welcome-demo routing and the component-showcase descriptor host.
