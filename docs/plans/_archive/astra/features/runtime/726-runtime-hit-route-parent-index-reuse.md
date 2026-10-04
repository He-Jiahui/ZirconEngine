---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-08-31-borrowed-dispatch-route-sharing.md
related_records:
  - docs/plans/astra/features/runtime/705-hit-route-publication-scratch-reuse.md
  - docs/plans/astra/features/runtime/723-runtime-index-output-capacity.md
  - docs/plans/astra/features/runtime/725-runtime-visibility-single-pass-publication.md
  - docs/plans/astra/features/runtime/727-runtime-hit-grid-entry-capacity.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/tree/hit_test/route_index.rs
tests:
  - zircon_runtime/src/ui/tree/hit_test/route_index.rs
  - tools/tests/test_runtime_ui_hit_route_index_performance_contract.py
---

# Runtime hit-route parent-index reuse

The full hit-route publication walk already resolves each node's parent index while validating
the iterative chain. Route composition now carries that resolved index in the traversal scratch
entry, so the full-build path does not perform a second `BTreeMap::get` for the same parent.
Incremental input patches retain the existing map lookup fallback because they do not share the
full-build traversal scratch. Route order, cycle and missing-parent rejection, inherited input
policy, pointer visibility, and the immutable shared route-table boundary are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime200 route publication | Reuse the parent index discovered during full-build validation when composing each route node. | RED/GREEN route source contract, existing deep/cycle/missing-parent regressions, scoped Rustfmt, and the batched Runtime/Editor source suite; managed Release allocation/latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

For a full publication with `N` arranged nodes, parent-index lookup work is reduced from up to two
ordered-map probes per visited node to one. The temporary chain remains iterative and is cleared
between components; no second route authority or persistent per-node cache is introduced. Patch
publication keeps its prior semantics and lookup path.

## Local evidence

- The route contract first failed against the old `Vec<usize>` chain and four-argument composer
  call, then passed after the parent-index hint was threaded through the full build.
- Existing deep-chain, missing-parent, cycle, failed-component, and input-patch regressions remain
  attached to the production module; the focused route contract now passes `9/9`.
- The focused route/visibility/activity source-contract batch passes `20/20` (including the
  subsequent hit-grid entry projection guard).
- The refreshed single-invocation non-tooling Runtime/Editor performance-plus-pressure batch
  covers `347` modules and passes `1333/1333` tests in `7.969s` after this source change and the
  subsequent hit-grid projection slice.
- Scoped production Rustfmt, `git diff --check`, and plan-record integrity checks pass.
- Current source snapshot SHA-256: `B1774D1AA3183F3E833D0E85F23C40D1B641DA070E6CCE4534FDE6C7A7D1FE00`
  (`route_index.rs`); the companion visibility publication remains
  `373DFC611BE250177E27F29055F49D7EDDE056DF2D62984EEB293053914D944D`.

This is source/contract evidence, not managed Cargo execution or product CPU, allocator, RSS, or
p50/p95/p99 acceptance evidence.

## Managed acceptance gate

This slice joins the deferred owner-attributed Runtime/Editor admission recorded in `696`. No new
coordinator request or status query is issued for this local change. Keep the row at
`implemented_pending_validation` until a managed Windows Release run measures full route
publication lookup work, allocation, and latency at the declared UI-node workloads.
