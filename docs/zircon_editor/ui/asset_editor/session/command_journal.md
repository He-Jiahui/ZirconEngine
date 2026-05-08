---
related_code:
  - zircon_editor/src/ui/asset_editor/session/journal.rs
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/ui/asset_editor/command.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
implementation_files:
  - zircon_editor/src/ui/asset_editor/session/journal.rs
  - zircon_editor/src/ui/asset_editor/command.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
plan_sources:
  - user: 2026-05-08 continue all UI milestones until complete
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
  - .codex/plans/UI 后续产品化与验证归档计划.md
tests:
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/asset_editor/command.rs zircon_editor/src/ui/asset_editor/undo_stack.rs zircon_editor/src/ui/asset_editor/session/mod.rs zircon_editor/src/ui/asset_editor/session/journal.rs zircon_editor/src/ui/asset_editor/session/replay_artifact.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/tests/editing/ui_asset_replay.rs
  - git diff --check -- zircon_editor/src/ui/asset_editor/command.rs zircon_editor/src/ui/asset_editor/undo_stack.rs zircon_editor/src/ui/asset_editor/session/mod.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/tests/editing/ui_asset_replay.rs
  - cargo test -p zircon_editor --lib ui_asset_editor_command_journal --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m9-replay-artifact --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# UI Asset Editor Command Journal

`journal.rs` is the M9 adapter from durable editor command records back into the existing UI Asset Editor command path. It does not add a second editing system. `UiAssetEditorSession::apply_command_journal(...)` validates a versioned journal, converts each journal command into `UiAssetEditorCommand`, and applies it through `apply_command_with_effects(...)` so undo, redo, source remap, document replay, and cross-file external effects stay on the established session path.

## Journal Contract

`UiAssetEditorCommandJournal` is versioned by `UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION`. Entries must have strictly increasing `sequence` values. Sequence validation happens before any entry is applied, so malformed journals do not partially mutate the session.

`UiAssetEditorJournalCommand` currently supports two replay surfaces:

- `SourceEdit`: stores a label and next source buffer, then replays as a `DocumentEdit` command so the undo stack keeps a useful label and document diff.
- `TreeEdit`: stores the structured tree edit, label, next source, optional selection, optional document replay bundle, and undo/redo external effects.

The command, tree-edit, document-replay, and external-effect DTOs derive serde traits for journal persistence. Unlike the bug-report artifact, this journal may contain raw source because it is the internal replay input. Public bug reports should use the redacted artifact export instead.

## Boundaries

The adapter is intentionally below host-event capture. It can replay a serialized command journal once another layer records editor operations, but it does not yet decide where journals are stored, how they are attached to bug reports, or how pointer/keyboard host events are normalized into command records.

## Validation

Formatting and whitespace checks passed on 2026-05-08 for the command journal slice. The focused `ui_asset_editor_command_journal` Cargo filter did not reach editor tests because `zircon_runtime` currently fails first in the active scene/ECS lane: `scene::ecs` is missing, and `SystemStage` / `Schedule` are no longer exported from `scene::components`. That blocker is outside this module.
