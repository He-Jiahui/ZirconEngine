---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/129-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
  - docs/plans/optimize/zircon_editor/181-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/716-hierarchy-filter-borrowed-query-capacity.md
  - docs/plans/astra/features/editor/719-hierarchy-filter-test-contract-repair.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/720-ecs-projection-node-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
related_code:
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
tests:
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
---

# Editor Hierarchy Filter Lazy Match Flags

## Scope

The retained hierarchy filter already skipped parent-index construction when a query had no
name matches, but it still allocated the full `Vec<bool>` used for ancestry propagation before
the scan. The projection now keeps the match flags in an `Option<Vec<bool>>` and materializes
that vector only when the first name match is found. A no-match query therefore performs the
same name scan and metrics publication without allocating match flags; matched queries retain
the existing ancestry, ordering, and output semantics.

## Plan completion list

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor129 / Editor181 / EUI-718 | Lazily materialize hierarchy match flags on the first name match while preserving the borrowed-query, ancestor, and exact-output paths from Editor716. | implemented_pending_validation | RED/GREEN source probes, the combined affected Runtime/Editor contract batch `150/150`, and scoped Rustfmt/diff checks pass; managed Rust and Release filter allocation/latency evidence remain pending. |

## Complexity and allocation boundary

The matched path remains `O(N)` for name scan plus one parent-index pass and one reverse ancestry
pass. The no-match path remains `O(N)` for matching but removes the `O(N)` bit-vector allocation
and exits before parent-index construction. The first match pays the same exact `Vec<bool>`
allocation as before; no second scan, index, or authority is introduced.

## Local evidence

- A source RED probe confirmed that the lazy flag branch was absent before implementation; the
  GREEN probe confirms `Option` storage and first-match `get_or_insert_with` materialization.
- The affected Runtime/Editor hierarchy, layout, text, and navigation contract batch passed
  `150/150` in one invocation.
- The refreshed non-tooling Runtime/Editor performance-plus-pressure loader covered 343 modules
  and passed `1320/1320` in `10.083s` after this change; the final current-source rerun passed
  `1320/1320` in `94.188s`.
- The in-file no-match source regression was then aligned with the `Option<Vec<bool>>`
  destructuring branch introduced here; the Editor product-interaction contract batch passed
  `10/10` after the repair.
- Scoped Rustfmt, `git diff --check`, and source/record whitespace checks pass. No tooling source
  was changed.
- These checks are source/contract evidence only and do not infer CPU, allocator, RSS, or
  p50/p95/p99 product performance.

## Managed acceptance gate

This slice joins the existing asynchronous Runtime/Editor admission recorded in `696`; no new
coordinator request or status query was issued. The record remains
`implemented_pending_validation` until an owner-attributed Windows Release run measures the
10k/100k hierarchy workloads and confirms the no-match allocation and input-to-present gates.
