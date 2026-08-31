---
title: Editor52 Showcase State Hash Cache
category: zircon_editor
report_id: Editor52-showcase-state-hash-cache-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor52 Showcase State Hash Cache

## Scope

This slice replaces the private component-showcase control-state cache with `HashMap`. Component
events update one control ID, while retained host projection repeatedly resolves explicit state for
each visible control; neither path traverses the cache or exposes its order.

Latest-state replacement, default-state fallback, event logging, component validation, and host
attribute projection are unchanged. Attribute and TOML map owners remain `BTreeMap`, preserving
stable output where order is material.

## Performance Workload

The release workload fills 16,384 realistic control IDs and performs 4,096 stable state lookups for
the final control.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered control-state lookups | 4,096 | 0 |
| Hash control-state lookups | 0 | 4,096 |
| State allocations on hits | 0 | 0 |
| Ordered attribute projections | unchanged | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR52_SHOWCASE_STATE_HASH_CACHE_BENCH_V1`. Acceptance requires hash lookup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826ce_showcase_state_hash_cache_preserves_latest_values` covers
  replacement, isolation, and stored component values.
- `optimization_batch_20260826ce_showcase_state_hash_cache_has_no_order_contract` locks the private
  hash owner while retaining ordered attribute maps.
- `optimization_batch_20260826ce_showcase_state_hash_cache_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor52 still owns truthful builtin-view capability routing, provider/resource dependencies,
template localization, internal-sample separation, and product-scale retained UI qualification.
This slice only converges the component-showcase state lookup cache.
