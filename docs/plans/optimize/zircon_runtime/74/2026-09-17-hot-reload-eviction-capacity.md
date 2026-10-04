---
title: Runtime74 hot-reload compile-cache eviction bounded capacity
category: zircon_runtime
report_id: Runtime793-hot-reload-eviction-capacity-2026-09-17
date: 2026-09-17
session_id: root-runtime-editor-async-optimization-20260917
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime793 · Hot-reload compile-cache eviction bounded capacity

## Scope

`UiAssetHotReloadPlan::evict_compile_cache` joins template-rebuild targets and
removed compiled assets before passing the borrowed target list to the compile
cache. Both input vectors are already materialized and their sum is a safe
upper bound, but the old path used `Vec::new()` and paid geometric growth on a
large hot-reload batch. This slice changes only that temporary target-vector
admission; eviction order, duplicate handling in the cache, and the public
eviction report remain unchanged.

## Implementation

- Added `compile_cache_eviction_capacity`, which combines the two input lengths
  with `saturating_add`.
- Used the helper for `Vec::with_capacity` before the existing two `extend`
  calls.
- Kept the target iterator types, category order, cache authority, and report
  semantics intact.

## Deterministic pressure model

For 16,384 joined targets, the zero-capacity doubling model reports 13 legacy
growth events and the admitted bound reports 0. The lower ignored marker
`RUNTIME793_HOT_RELOAD_EVICTION_CAPACITY_BENCH_V1` records this comparison.
These are deterministic allocation-shape signals, not CPU, RSS, allocator, or
product hot-reload p50/p95/p99 measurements.

## TDD and local evidence

The Python source contract was intentionally RED while `evict_compile_cache`
still used `Vec::new()`, then GREEN at `3/3` after the saturating helper and
lower-module wiring were added. The lower module checks empty, ordinary, and
overflow bounds and carries the ignored Release marker. Scoped Rustfmt and the
merged Runtime/Editor source-contract and regression batches pass `556/1987`
performance-contract checks in `5.867s` and `920/3737` broad non-tooling
regression checks in `445.407s`, with zero failures, errors, or skips. The
final post-Runtime794 receipts supersede those baselines: `557/1990`
performance-contract checks in `71.984s` and `921/3740` broad non-tooling
regression checks in `882.184s`. These local source/model receipts are
recorded in the linked Astra completion ledgers.

## Source files

- `zircon_runtime/src/ui/template/asset/hot_reload_plan.rs`
- `zircon_runtime/src/ui/template/asset/hot_reload_plan/eviction_capacity_tests.rs`
- `tools/tests/test_runtime_hot_reload_eviction_capacity_performance_contract.py`

## Managed acceptance gate

Keep this slice `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Runtime tree,
runs the lower regression and ignored marker, and supplies allocator plus
hot-reload eviction p50/p95/p99 evidence. No standalone Cargo command or
coordinator status query is used here; tooling production work remains deferred
for the later Rust migration.
