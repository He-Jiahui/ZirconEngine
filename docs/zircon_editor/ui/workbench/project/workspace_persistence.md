---
related_code:
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_load.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_document.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_editor/src/ui/workbench/project/project_editor_workspace.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
implementation_files:
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_load.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - cargo test -p zircon_editor --lib project_open_with_corrupt_workspace_falls_back_to_global_layout_with_diagnostic --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed, 1 passed, 0 failed, 2023 filtered out)
  - cargo test -p zircon_editor --lib editor_project_document_ignores_unknown_workspace_format_with_diagnostic --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed, 1 passed, 0 failed, 2024 filtered out)
  - cargo test -p zircon_editor --lib editor_project_document_roundtrips_world_and_workspace --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed, 1 passed, 0 failed, 2024 filtered out)
  - cargo test -p zircon_editor --lib startup_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed, 2 passed, 0 failed, 2024 filtered out)
  - cargo test -p zircon_editor --lib create_project_and_open_persists_recent_project_and_returns_project_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed, 1 passed, 0 failed, 2025 filtered out)
doc_type: module-detail
---

# Project Workspace Persistence

Project workspace persistence is the editor-only layer stored next to a runtime project at `.zircon/editor-workspace.json`. It captures `ProjectEditorWorkspace`: the current `WorkbenchLayout`, open view instances, active center tab, and active drawers. Runtime scene loading remains independent of this file.

`EditorUiHost::open_project(...)` first asks `ProjectAuthority` to prepare the project, publishes it through the Runtime `AssetManager`, and retrieves the retained `ProjectManager` generation. Only then does `EditorProjectDocument::load_from_activated_project(&project, project_info, settings, allows_scene_restore)` load the default scene and editor workspace using the same active `EditorContext` settings authority and lifecycle policy. The direct project loader is test-only; production no longer owns a parallel path-based loader or reopens the project from a raw filesystem path. A missing workspace file is the normal first-open case and returns `editor_workspace = None` without a diagnostic. An unreadable or corrupt workspace, or one with an unsupported `format_version`, still lets the project open with `editor_workspace = None` and records an `EditorWorkspaceRestoreDiagnostic` containing the path and message.

Explicit project open uses that diagnostic to present a clear status line: the project opened, but layout restore fell back to the default chain. No-request startup does not reopen a remembered project; it remains in the Welcome chooser until an explicit project launch has passed the project lifecycle policy. Applying `None` through `EditorManager::apply_project_workspace(...)` rebuilds the default session, which in turn honors the saved global default layout before falling back to the Material/Fyrox/JetBrains/Unreal default workbench.

Preset switching is still handled separately through `LayoutCommand::SavePreset` and `LayoutCommand::LoadPreset`. Project-local preset assets remain under `assets/editor/layout-presets`, while the workspace file represents the automatic per-project editor session state.
