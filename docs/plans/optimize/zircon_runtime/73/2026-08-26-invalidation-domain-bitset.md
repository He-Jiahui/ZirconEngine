---
title: Runtime73 Invalidation Domain Bitset
category: zircon_runtime
report_id: Runtime73-invalidation-domain-bitset-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime73 Invalidation Domain Bitset

## Scope

This slice replaces the commit-local `BTreeSet` used to accumulate seven invalidation domains with
a `u8` bitset. Each public `UiInvalidationChange` retains its ordered reason set, transaction
changes remain keyed by ordered node ID, and the commit still publishes changed nodes in ascending
node order.

After accumulation, the bitset is traversed through an explicit `UiInvalidationReason::ALL` array
in the enum's previous order. Each touched domain generation therefore advances exactly once, with
unchanged dirty flags and serialized commit shape.

## Performance Workload

The release workload accumulates three reasons for each of 65,536 changed-node rows.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered domain insertions | 196,608 | 0 |
| Domain bit tests/sets | 0 | 196,608 |
| Ordered changed-node projection | unchanged | unchanged |
| Public per-change reason sets | unchanged | unchanged |

The ignored release gate runs 21 alternating sample pairs and emits
`RUNTIME73_INVALIDATION_DOMAIN_BITSET_BENCH_V1`. Acceptance requires bitset accumulation P95 to be
at least 30% below the legacy `BTreeSet` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826cl_runtime_invalidation_domain_bitset_preserves_commit_contract`
  covers changed-node order and exactly-once generation advancement across merged reasons.
- `optimization_batch_20260826cl_runtime_invalidation_touched_domains_use_fixed_bitset` locks the
  seven-domain bitset and explicit ordered traversal.
- `optimization_batch_20260826cl_runtime_invalidation_domain_bitset_release_benchmark` reports
  paired release P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime73 still owns style invalidation provenance, selector and token dependency indexes,
transition lifecycle, theme cutover, and product-scale retained-UI qualification. This slice only
converges commit-local domain accumulation.
