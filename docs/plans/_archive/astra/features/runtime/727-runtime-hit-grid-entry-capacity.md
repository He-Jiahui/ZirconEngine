---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-10-default-scroll-candidate-scratch.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
related_records:
  - docs/plans/astra/features/runtime/681-hit-test-stacked-output-capacity.md
  - docs/plans/astra/features/runtime/723-runtime-index-output-capacity.md
  - docs/plans/astra/features/runtime/726-runtime-hit-route-parent-index-reuse.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/tree/hit_test.rs
tests:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - tools/tests/test_runtime_ui_hit_route_index_performance_contract.py
---

# Runtime hit-grid entry projection capacity

The hit-grid builder now reserves the `draw_order` length before projecting stable geometry
entries and reuses the arranged-node index resolved for each draw-order ID when selecting its
route entry. Each draw-order node can contribute at most one `UiHitTestEntry`, while malformed,
non-pointer, or invalid-route nodes are still skipped through the same fail-closed predicates.
The explicit loop preserves draw order and filtering semantics, removes geometric growth of the
common entry buffer, and avoids a duplicate ordered-map probe; route publication, cell bounds,
sorting, and query behavior are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime11C hit-grid publication | Reserve the authoritative draw-order upper bound and reuse its arranged-node index before filtering. | RED/GREEN source contract, existing hit-grid behavior/pressure coverage, scoped Rustfmt, and the batched Runtime/Editor source suite; managed Release allocation/latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

The projection remains linear in `draw_order`. The reserved buffer is an upper bound, not an
assumption that every node survives geometry, route, or pointer filtering; the existing arranged
index lookup is carried into route-index conversion instead of probing the ordered map twice. No
additional route or cell authority is introduced, and sorting/bounded cell membership retain
their existing behavior.

## Local evidence

- The new source contract first failed against the `filter_map().collect()` implementation and
  the duplicate route-index lookup, then passed after the explicit
  `Vec::with_capacity(arranged_tree.draw_order.len())` projection and single-index loop were
  installed.
- The focused route/visibility/activity contract batch passes `20/20` and the refreshed unified
  non-tooling Runtime/Editor performance-plus-pressure batch covers `347` modules with
  `1333/1333` tests passing in `7.969s`.
- Scoped production Rustfmt, `git diff --check`, and plan-record integrity checks pass.
- Module-boundary review keeps the small projection loop beside `build_hit_grid`, its existing
  owner; route, geometry, query-scratch, and cell-membership responsibilities remain in their
  dedicated child modules, so no unrelated large-file reorganization is needed.
- Current source snapshot SHA-256: `55890A9D9FEB3A5BFD98BE5729E50FD294D2435CD58EDF0E565E3496027E6041`
  (`hit_test.rs`).

This is source/contract evidence, not managed Cargo execution or product CPU, allocator, RSS, or
p50/p95/p99 acceptance evidence.

## Managed acceptance gate

This slice joins the deferred owner-attributed Runtime/Editor admission recorded in `696`. No new
coordinator request or status query is issued for this local change. Keep the row at
`implemented_pending_validation` until a managed Windows Release run measures hit-grid entry
publication allocations and latency at the declared UI-node workloads.
