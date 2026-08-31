---
title: Editor13 Layout Host Hash Placements
category: zircon_editor
report_id: Editor13-layout-host-hash-placements-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Layout Host Hash Placements

## Scope

This slice replaces layout host collection's private `BTreeMap` with `HashMap`. The two production
consumers use the result for membership, keyed host updates, or conversion into `HashSet`; neither
observes map iteration order.

Document-tree traversal remains depth-first and duplicate view IDs retain latest-placement-wins
semantics. Drawer, document path, floating-window, and exclusive-page host values are unchanged.

## Performance Workload

The release workload collects 16,384 realistic view-instance keys and performs 4,096 placement
lookups per iteration.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered placement insertions | 16,384 | 0 |
| Ordered placement lookups | 4,096 | 0 |
| Hash placement entries | 0 | 16,384 |
| Hash placement lookups | 0 | 4,096 |
| Order projections | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR13_LAYOUT_HOST_HASH_PLACEMENTS_BENCH_V1`. Acceptance requires hash build-and-lookup P95 to
be at least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ch_layout_host_hash_placements_preserve_last_host` covers split-tree
  paths, unique views, and duplicate latest-placement behavior.
- `optimization_batch_20260826ch_layout_host_hash_placements_have_no_order_projection` locks both
  hash owners and prevents a hidden sort boundary from returning.
- `optimization_batch_20260826ch_layout_host_hash_placements_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor13 still owns layout schema migration, window recovery, transactionality, focus restoration,
multi-monitor placement, and product-scale docking qualification. This slice only converges the
ephemeral view-instance-to-host lookup map.
