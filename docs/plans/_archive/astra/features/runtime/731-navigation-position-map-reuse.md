---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-position-map-reuse.md
related_records:
  - docs/plans/astra/features/runtime/665-navigation-rebuild-candidate-stream.md
  - docs/plans/astra/features/runtime/666-navigation-group-candidate-single-pass.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/navigation_index.rs
tests:
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
  - tools/tests/test_runtime_ui_navigation_index_performance_contract.py
---

# Runtime navigation position-map reuse

The retained navigation index kept sorted tab candidate vectors in deterministic
`BTreeMap`/`Vec` containers, but rebuilt a `BTreeMap<UiNodeId, usize>` position
map for every scope. Position lookup is a read-only hot path and does not depend
on map iteration order. The position indexes now use `HashMap` for expected
constant-time lookup, while stable outer and inner buckets are retained across
rebuilds. Candidate ordering, sorting, modal scope selection, and wraparound
semantics are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A/P1-14 navigation tab lookup | Reuse lookup-only position maps with `HashMap`; retain nested scope buckets and prune removed scopes during rebuild. | RED/GREEN source contract, in-file capacity/pruning regression, navigation contract `7/7`, scoped Rustfmt, and the batched non-tooling Runtime/Editor suite (`351` modules, `1347/1347` tests) pass; managed Cargo/Release allocation and navigation-latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

For a candidate list of length `F`, rebuilding a position map remains `O(F)`.
Subsequent tab navigation lookups are expected `O(1)` instead of ordered-tree
lookup. The base map is cleared in place; group and MUI-root maps retain buckets
for still-present scopes, clear their inner entries in place, and allocate only
when a scope or capacity is genuinely new. Removed scopes are pruned. Sorted
candidate vectors remain the ordering authority, so `HashMap` iteration order
cannot change navigation behavior.

## Local evidence

- The source contract was intentionally RED while position maps used
  `BTreeMap` and a fresh `position_map` collector, then GREEN after the
  lookup-only `HashMap` helpers and retained nested buckets were installed.
- The in-file regression rebuilds a stable group with a smaller candidate set,
  verifies retained outer/inner capacity, confirms stale node IDs are removed,
  and confirms an absent group is pruned.
- `tools/tests/test_runtime_ui_navigation_index_performance_contract.py`
  passes all `7/7` checks.
- `rustfmt --edition 2021 --check --config skip_children=true` passes for the
  owned navigation-index root; child test-module formatting drift predates this
  slice and was not reformatted.
- The final same-source single-process non-tooling Runtime/Editor
  performance-plus-pressure loader covered `351` modules and passed
  `1347/1347` tests in `64.780s` with zero failures or errors (earlier warm
  runs completed in `10.868s` and `121.072s`).
- Current source SHA-256 values are
  `3C227035118A548A0BFB7F8BBAB11772BBB3BCB772FC249A5BBCE9B1BC2408DE`
  (`navigation_index.rs`),
  `9555A4A3B75550CD3F337E152D163632DBEC7E3FB7E626242F8AF0C6C4ACE80E`
  (`navigation_index/tests.rs`), and
  `635F40FECB0B38875B2BA0B74BAB3565F188871AA90001E2820FDD57CCDE0963`
  (`test_runtime_ui_navigation_index_performance_contract.py`).

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

This slice joins the existing owner-attributed Runtime/Editor admission recorded
in `696`. No new coordinator request or status query was issued. Keep the row at
`implemented_pending_validation` until a managed Windows Release batch verifies
compilation, ordered-candidate parity, allocation behavior, and navigation
latency under the declared node workloads.
