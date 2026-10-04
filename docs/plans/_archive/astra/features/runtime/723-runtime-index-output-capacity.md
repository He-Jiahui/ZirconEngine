---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/705-hit-route-publication-scratch-reuse.md
  - docs/plans/astra/features/runtime/706-accessibility-visibility-detached-scratch-reuse.md
  - docs/plans/astra/features/runtime/725-runtime-visibility-single-pass-publication.md
  - docs/plans/astra/features/runtime/726-runtime-hit-route-parent-index-reuse.md
  - docs/plans/astra/features/runtime/727-runtime-hit-grid-entry-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/tree/hit_test/route_index.rs
  - zircon_runtime/src/ui/surface/arranged_visibility.rs
tests:
  - zircon_runtime/src/ui/tree/hit_test/route_index.rs
  - zircon_runtime/src/ui/surface/arranged_visibility.rs
  - tools/tests/test_runtime_ui_hit_route_index_performance_contract.py
  - tools/tests/test_runtime_ui_arranged_visibility_index_performance_contract.py
---

# Runtime hit-route and visibility index output capacity

Two retained Runtime UI publication paths now reserve their known output
bounds before filling them. Hit-route publication builds its route-node table
with `arranged_tree.nodes.len()` capacity, while arranged-visibility rebuild
reserves the published `node_ids` count after clearing the retained buffer.
Ordering, fail-closed handling, route semantics, visibility bits, clone
behavior, and the existing scratch-reuse boundaries are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime200 hit-route and visibility publication | Pre-size the route-node output and retained visibility node-id output from authoritative counts. | RED/GREEN source probes, production capacity regressions, route/visibility contract checks, and the batched Runtime/Editor suite pass; managed Release allocation/latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

Both paths remain linear in the arranged node count. The changes remove
geometric growth of vectors whose lower bounds are already known; they do not
add a second tree/index authority or change the published route and visibility
representations. Existing bounded scratch retention continues to govern deep
or malformed trees.

## Local evidence

- RED probes confirmed the two explicit capacity paths were absent before the
  patch; GREEN probes and in-file tests now assert the route table and node-id
  outputs retain the known lower bounds.
- The final single-invocation non-tooling Runtime/Editor performance-plus-
  pressure batch covered 347 modules and passed `1331/1331` tests in
  `6.361s` after this slice; the focused route/visibility contract batch
  passed `18/18`, and the broader route/visibility/surface targeted batch
  passed `150/150` across 31 modules in `33.702s`.
- Scoped Rustfmt and `git diff --check` pass; record whitespace/reference
  checks pass, and no tooling production source is changed.
- Current source snapshot SHA-256: `865C0154113352CED65EC98324D3928A23961B7945C65B1E390227271B31698C`
  (`route_index.rs` before the follow-up parent-index hint) and
  `373DFC611BE250177E27F29055F49D7EDDE056DF2D62984EEB293053914D944D`
  (`arranged_visibility.rs` after Runtime725).
- Runtime726's follow-up route source snapshot is recorded in
  `726-runtime-hit-route-parent-index-reuse.md`; this record intentionally retains the
  pre-follow-up hash as the capacity-slice baseline.

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

This slice joins the asynchronous owner-attributed Runtime/Editor admission
recorded in `696`; because that admission was rejected before ticket creation,
no new coordinator request or status query is issued for this local slice. It
must be included in the next owner-attributed batch. Keep the row at
`implemented_pending_validation` until a managed Windows Release run measures
route/visibility publication allocations and latency at the declared UI-node
workloads.
