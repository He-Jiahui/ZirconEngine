---
title: Runtime74 hot-reload template-asset projection bounded capacity
category: zircon_runtime
report_id: Runtime794-hot-reload-template-assets-capacity-2026-09-17
date: 2026-09-17
session_id: root-runtime-editor-async-optimization-20260917
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime794 · Hot-reload template-asset projection bounded capacity

## Scope

`template_assets_for_surface` filters the concatenation of template-rebuild and
removed-compiled asset targets for each active surface. The two plan vectors
are already materialized, so their combined length is a safe upper bound for
the filtered output. This slice admits that bound before the existing filter
and clone sequence without changing membership, order, or surface ownership.

## Implementation

- Added `template_asset_capacity` with saturating addition of the two input
  lengths.
- Replaced the zero-capacity collect with a bounded vector followed by the same
  chained filter/cloned iterator and `extend`.
- Kept target category order, duplicate behavior, surface-index lookup, and
  returned asset strings unchanged.

## Deterministic pressure model

For 16,384 joined targets, the zero-capacity doubling model reports 13 legacy
growth events and the admitted upper bound reports 0. The lower ignored marker
`RUNTIME794_HOT_RELOAD_TEMPLATE_ASSETS_CAPACITY_BENCH_V1` records this shape
comparison. It is allocation evidence only, not CPU, RSS, allocator, or
product hot-reload p50/p95/p99 evidence.

## TDD and local evidence

The Python source contract was RED while the original `collect()` path had no
capacity helper, then GREEN at `3/3` after the helper, bounded `extend`, and
lower-module wiring were added. The lower module checks empty, ordinary, and
overflow-safe bounds and carries the ignored Release marker. Final batched
Runtime/Editor source-contract and regression receipts are `557/1990` in
`71.984s` and `921/3740` in `882.184s`, respectively, with zero failures,
errors, or skips; they are recorded in the linked Astra ledgers.

## Source files

- `zircon_runtime/src/ui/template/asset/hot_reload_executor.rs`
- `zircon_runtime/src/ui/template/asset/hot_reload_executor/template_assets_capacity_tests.rs`
- `tools/tests/test_runtime_hot_reload_template_assets_capacity_performance_contract.py`

## Managed acceptance gate

Keep this slice `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Runtime tree,
runs the lower regression and ignored marker, and supplies allocator plus
hot-reload template projection p50/p95/p99 evidence. No standalone Cargo
command or coordinator status query is used here; tooling production work
remains deferred for the later Rust migration.
