---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-25-pane-projection-generation-cache-design.md
  - docs/plans/optimize/zircon_editor/129-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
  - docs/plans/optimize/zircon_editor/01/2026-09-13-hierarchy-projection-single-pass.md
related_records:
  - docs/plans/astra/features/editor/716-hierarchy-filter-borrowed-query-capacity.md
  - docs/plans/astra/features/editor/718-hierarchy-filter-lazy-match-flags.md
  - docs/plans/astra/features/editor/719-hierarchy-filter-test-contract-repair.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/731-hierarchy-index-rebuild-reuse.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/hierarchy_projection.rs
tests:
  - tools/tests/test_editor_hierarchy_projection_single_pass_contract.py
---

# Editor hierarchy projection single-pass selection state

The retained hierarchy pane projection first scanned every logical row to find
whether any row was selected, then scanned the same rows again to construct the
host `SceneNodeData` model. The required DTO pass now tracks `has_selection`
while constructing `hierarchy_nodes`, and the collected rows are passed directly
to the host model. Row identity, ordering, depth conversion, selection values,
and the template-state fallback remain unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 hierarchy/pane projection | Merge selection-state discovery into the required scene-row projection, removing the duplicate payload traversal. | RED/GREEN source contract, retained-row regression contract, focused Runtime/Editor batch, and scoped Rustfmt pass; managed Cargo/Release CPU/allocation evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

For a hierarchy payload of `N` rows, the previous worst case visited the source
rows once for selection and once for DTO construction (`2N` visits). The current
path visits the source rows once (`N` visits) and reuses the resulting vector;
the downstream model still owns the same one `SceneNodeData` per row and keeps
the same `O(N)` row materialization requirement. No cache key, generation, route,
or filtering semantics changed.

## Local evidence

- The focused contract was intentionally RED against the old `.any(|node|
  node.selected)` plus second `.map` path, then GREEN after the single-pass
  projection was introduced.
- The contract also requires the one collected `hierarchy_nodes` vector to be
  handed to `model_rc`, preventing a second source iterator from returning.
- Current source SHA-256:
  `08B7860BBC79880E60B9AAC902A6EC74CFD5785898EC888971205F984BE522D4`
  (`hierarchy_projection.rs`), and test SHA-256
  `B1921F97157534BB32DD838D873C5115E0C0BB8A7D869A57AD84E8858DDF5A9D`.
- The focused new contracts pass `4/4`; the single-process non-tooling
  Runtime/Editor performance-plus-pressure batch covered 351 modules and passed
  `1345/1345` tests in `39.080s`.
- A subsequent same-source rerun of that shared non-tooling batch covered 351
  modules and passed `1346/1346` tests in `8.890s`.

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

The existing owner-attributed Windows Release admission remains deferred by the
external dirty `E:\Git\zr_vm` worktree gate. No new coordinator request or
status query was issued for this slice. Keep this record at
`implemented_pending_validation` until the hierarchy projection allocation and
latency gates are measured in the next batched validation.
