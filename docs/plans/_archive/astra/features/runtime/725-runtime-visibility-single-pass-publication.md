---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/astra/features/runtime/32-arranged-visibility-scratch-reuse.md
related_records:
  - docs/plans/astra/features/runtime/723-runtime-index-output-capacity.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/726-runtime-hit-route-parent-index-reuse.md
  - docs/plans/astra/features/runtime/727-runtime-hit-grid-entry-capacity.md
implementation_files:
  - zircon_runtime/src/ui/surface/arranged_visibility.rs
tests:
  - zircon_runtime/src/ui/surface/arranged_visibility.rs
  - tools/tests/test_runtime_ui_arranged_visibility_index_performance_contract.py
---

# Runtime arranged-visibility single-pass publication

`UiArrangedVisibilityIndex::rebuild` already receives the authoritative sorted
`BTreeMap<UiNodeId, usize>`, but previously copied its keys and then performed
another map lookup for every published node. Publication now reserves the same
known output size and consumes each `(node_id, arranged_index)` pair once,
writing the sorted node ID and compact visibility bit in that pass. Invalid
arranged indices still fail closed, and ordering, visibility inheritance,
clone behavior, and the retained scratch boundary are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime200 visibility publication | Merge sorted node-ID publication and visibility-bit projection into one authoritative index traversal. | RED/GREEN source probe, production ordering/visibility regression, focused visibility contracts, and the batched Runtime/Editor suite pass; managed Release allocation/latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

The publication phase remains linear in the published node count. It removes
the second key traversal and per-node `BTreeMap::get` (`O(N log N)`) while
retaining the map's sorted iteration order and the existing compact bitset.
Resolver work and malformed-tree fail-closed behavior are unchanged.

## Local evidence

- The RED probe confirmed the old key-copy plus per-node lookup shape; the
  GREEN source guard and in-file regression require one
  `node_indices.iter().enumerate()` publication pass.
- Current source snapshot SHA-256: `373DFC611BE250177E27F29055F49D7EDDE056DF2D62984EEB293053914D944D`
  (`arranged_visibility.rs`).
- The current combined non-tooling Runtime/Editor performance-plus-pressure
  batch covers 347 modules and passes `1333/1333` tests in `7.969s`; the focused
  route/visibility/activity contracts remain green at `20/20` after the linked
  Runtime726/Runtime727 follow-ups (including single-index hit-grid projection).
- Scoped production Rustfmt, `git diff --check`, Python compilation, and plan
  record whitespace/reference checks pass.

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

This slice joins the deferred owner-attributed Runtime/Editor admission in
`696`. No new coordinator request or status query is issued for this local
change. Keep the row at `implemented_pending_validation` until a managed
Windows Release run measures visibility publication allocations and latency at
the declared UI-node workloads.
