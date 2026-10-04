status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md

# Asset drag visibility

## Current source and repair

The asset drag predicate is consumed by the retained-host app boundary. Its
definition now uses `pub(in crate::ui::retained_host::app)`, while the local
module keeps its existing `pub(super)` re-export. This matches the consumer
boundary without widening the module root API.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M36 | Restore retained-host asset drag predicate visibility | implemented_pending_validation | Scoped rustfmt and diff checks passed; combined Runtime/Editor batch pending |

This is a visibility-only repair and carries no independent performance claim.
