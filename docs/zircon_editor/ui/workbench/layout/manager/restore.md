---
related_code:
  - zircon_editor/src/ui/host/builtin_layout/hybrid_layout.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_editor/src/ui/workbench/layout/manager/defaults.rs
  - zircon_editor/src/ui/workbench/layout/manager/restore.rs
  - zircon_editor/src/ui/workbench/layout/restore_policy.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
implementation_files:
  - zircon_editor/src/ui/workbench/project/editor_project_document.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/workbench/layout/manager/restore.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/ui/workbench/layout/manager/restore.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - cargo test -p zircon_editor --lib project_open_with_corrupt_workspace_falls_back_to_global_layout_with_diagnostic --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed)
  - cargo test -p zircon_editor --lib editor_project_document_ignores_unknown_workspace_format_with_diagnostic --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-16: passed)
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

Project workspace snapshots are a recoverable source, not a hard dependency for project loading. `.zircon/editor-workspace.json` is read after the runtime project and scene are opened; corrupt JSON, read errors, or an unsupported workspace `format_version` produce an `EditorWorkspaceRestoreDiagnostic` and continue with `editor_workspace = None`. Startup then reports the diagnostic in the status line and applies the same global-default/default fallback chain.
