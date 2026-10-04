---
title: Runtime navigation position-map reuse
category: zircon_runtime
report_id: Runtime11A-navigation-position-map-reuse-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A navigation position-map reuse

## Scope

`UiSurfaceNavigationIndex::next_tab_target` already reads pre-sorted candidate
vectors, but each rebuild recreated ordered position maps for the base, modal
group, and MUI-root scopes. Those maps are lookup-only; ordering and iteration
semantics belong to the existing sorted vectors and `BTreeMap` scope indexes.

## Implementation

The three position-map layers now use `HashMap`. Rebuild helpers clear and
refill the base map in place, retain outer group/root buckets whose scopes are
still present, clear and reuse their inner maps, and prune stale scopes. The
candidate vectors and all sorting/comparison code remain unchanged. This keeps
the deterministic navigation order while removing ordered-tree lookup overhead
and avoidable steady-state bucket allocation.

## Deterministic work model

For a scope with `F` tab candidates, position-map rebuild remains `O(F)` and
navigation lookup is expected `O(1)` rather than `O(log F)`. After the first
build, stable scope IDs reuse their outer and inner hash-table capacity; only
new scopes or genuine capacity growth allocate. Candidate sorting remains a
rebuild-only cost and is intentionally outside this lookup-map change.

## Validation

- The navigation source contract was RED against `BTreeMap` position maps and
  the old fresh collector, then GREEN after the reusable `HashMap` helpers were
  installed.
- The lower Rust regression source verifies stable bucket capacity, stale-node
  pruning, and stale-scope removal. It has not been executed through managed
  Cargo because the existing external worktree admission is deferred.
- The focused navigation performance contract passes `7/7`.
- Scoped Rustfmt passes for the owned root with `skip_children=true`; unrelated
  child-module formatting drift remains untouched.
- The final same-source single-process non-tooling Runtime/Editor
  performance-plus-pressure batch covered `351` modules and passed
  `1347/1347` tests in `64.780s` (earlier warm runs completed in `10.868s`
  and `121.072s`).
- Current source SHA-256 values:

  | File | SHA-256 |
  | --- | --- |
  | `zircon_runtime/src/ui/surface/navigation_index.rs` | `3C227035118A548A0BFB7F8BBAB11772BBB3BCB772FC249A5BBCE9B1BC2408DE` |
  | `zircon_runtime/src/ui/surface/navigation_index/tests.rs` | `9555A4A3B75550CD3F337E152D163632DBEC7E3FB7E626242F8AF0C6C4ACE80E` |
  | `tools/tests/test_runtime_ui_navigation_index_performance_contract.py` | `635F40FECB0B38875B2BA0B74BAB3565F188871AA90001E2820FDD57CCDE0963` |

These are source/contract results only. Managed Cargo, allocator, RSS, and
navigation p50/p95/p99 product evidence remain pending.

## Acceptance boundary

Keep `validation_status: managed_validation_pending` until the existing
owner-attributed Windows Release batch verifies compile, candidate-order parity,
allocation behavior, and tab-navigation latency. No new coordinator request or
status query was made for this slice.
