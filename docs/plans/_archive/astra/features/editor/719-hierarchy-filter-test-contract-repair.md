---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/129-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
  - docs/plans/optimize/zircon_editor/181-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/716-hierarchy-filter-borrowed-query-capacity.md
  - docs/plans/astra/features/editor/718-hierarchy-filter-lazy-match-flags.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
tests:
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
---

# Editor hierarchy-filter lazy-flag test contract repair

The lazy match-flag optimization in Editor718 replaced the old explicit
`if name_match_count == 0` branch with an `Option<Vec<bool>>` destructuring
branch. The in-file source regression still searched for the removed string,
which would fail when the Editor crate tests compiled even though the
production behavior was correct. The regression now anchors on the actual
`let Some(mut included) = included else` no-match exit and continues to verify
that it precedes parent-index construction and returns an empty projection.

## Plan completion list

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor129 / Editor181 / EUI-719 | Keep the hierarchy source/behavior regression aligned with the Editor718 lazy allocation contract. | implemented_pending_validation | Editor product-interaction contract `10/10`, Rustfmt, diff, and source-order checks pass; managed Editor Cargo and Release evidence remain pending. |

## Boundary

This is a test-contract-only repair. It does not change hierarchy matching,
ancestor retention, row ordering, metrics, or allocation behavior. The managed
Windows gate remains the same owner-attributed validation recorded in Runtime696.
