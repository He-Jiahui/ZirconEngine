status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md

# Editor owner import repairs

## Current source and repair

Several Editor modules had declarations and imports pointing at a former
owner. Durable journal discovery models are owned by `durable::model`, hit
testing exports its pointer-move predicate from `surface_frame_builder`, and
style/chrome/badge helpers are re-exported from their actual parent modules.
Asset Activity layout data uses the Editor-owned `SharedString` alias instead
of a direct Slint type dependency. These changes restore existing module
boundaries without changing runtime behavior.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M34 | Repair journal, hit-test, chrome, badge, style, and Asset Activity owner imports | implemented_pending_validation | Scoped rustfmt/diff checks passed; combined Editor batch pending |

This is a compile-structure repair and carries no independent performance
claim.
