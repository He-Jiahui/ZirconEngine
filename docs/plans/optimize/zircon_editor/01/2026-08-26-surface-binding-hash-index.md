---
title: Editor01 Surface Binding Hash Index
category: zircon_editor
report_id: Editor01-surface-binding-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Surface Binding Hash Index

## Scope

This slice gives the asset, inspector, pane, and welcome retained-surface bridges private
`HashMap` binding indexes. Every control event resolves a route binding ID and performs one point
lookup; these bridges expose no binding-map iteration or serialization contract.

The shared lookup helper now accepts both ordered and hash indexes. Existing workbench bridges keep
their `BTreeMap` owner untouched, preserving their current construction and any ordered diagnostic
surface while the four isolated surface hot paths avoid tree lookup.

## Performance Workload

The release workload builds a 16,384-entry realistic route-binding index and performs 4,096
binding-ID lookups per iteration.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered surface binding insertions | 16,384 | 0 |
| Ordered surface binding lookups | 4,096 | 0 |
| Hash surface binding entries | 0 | 16,384 |
| Hash surface binding lookups | 0 | 4,096 |
| Workbench ordered owner | unchanged | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_SURFACE_BINDING_HASH_INDEX_BENCH_V1`. Acceptance requires hash build-and-lookup P95 to be
at least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ci_surface_binding_hash_index_matches_ordered_lookup` covers hit and
  miss parity across ordered and hash binding indexes.
- `optimization_batch_20260826ci_surface_binding_hash_index_keeps_workbench_order_owner` locks the
  dual-container helper, surface hash builder, four hash bridge owners, and ordered workbench
  builder.
- `optimization_batch_20260826ci_surface_binding_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained dispatch batching, route generation, invalidation, accessibility,
multi-window lifecycle, and product-scale input latency. This slice only converges four private
surface binding lookup tables.
