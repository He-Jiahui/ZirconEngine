status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md

# Core editing exports

## Current source and repair

The Editor host controller now imports `EditCommandError`, `EditorCommand`, and
`HistoryContextId` from their owning `command` and `engine` modules. The root
module keeps its child-only ownership boundary; no compatibility re-exports
were added.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M37 | Restore crate-local core editing command and history exports | implemented_pending_validation | Scoped diff check passed; combined Runtime/Editor batch pending |

This is a module wiring repair and carries no independent performance claim.
