---
title: Editor01 UI Router Hash Index
category: zircon_editor
report_id: Editor01-ui-router-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 UI Router Hash Index

## Scope

This slice replaces the editor UI router's exact-path owner with `HashMap`. Every retained UI event
dispatch now resolves its `(view, control, event kind)` path through expected constant-time lookup.

The router exposes no route iterator. Handler vectors remain the sole order owner, so multiple
handlers registered for one exact path continue to execute in registration order. Binding payload,
event normalization, unmatched-route behavior, and handler ownership are unchanged.

## Performance Workload

The release workload fills 4,096 long shared-prefix `UiEventPath` keys and performs 4,096 stable
hits for the final path.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered exact-route lookups | 4,096 | 0 |
| Hash exact-route lookups | 0 | 4,096 |
| Handler-order policy changes | 0 | 0 |
| Allocations on route hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_UI_ROUTER_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at least 30%
below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bx_ui_router_hash_index_preserves_handler_order` covers exact
  dispatch, unrelated route isolation, and same-path handler order.
- `optimization_batch_20260826bx_ui_router_hash_index_has_no_ordered_route_iteration` locks the
  unordered exact-route owner contract.
- `optimization_batch_20260826bx_ui_router_hash_index_p95` reports paired release P50/P95 samples
  and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges exact UI event
route lookup.
