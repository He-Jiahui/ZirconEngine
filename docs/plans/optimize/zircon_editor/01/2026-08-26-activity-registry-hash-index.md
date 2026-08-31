---
title: Editor01 Activity Registry Hash Index
category: zircon_editor
report_id: Editor01-activity-registry-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Activity Registry Hash Index

## Scope

This slice replaces the activity-view and activity-window registries with `HashMap` owners. UI
host reflection and command routing now resolve activity descriptors through expected
constant-time lookup. Registration uses the entry API so duplicate admission no longer performs a
second hash probe.

The public snapshot methods still clone their results and now sort explicitly by `view_id` or
`window_id`, preserving the deterministic ordering previously supplied implicitly by `BTreeMap`.
Duplicate errors, descriptor ownership, event routing, subscriptions, and reflection requests are
unchanged.

## Performance Workload

The release workload fills 512 activity IDs with long shared prefixes and performs 4,096 stable
hits for the final entry.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered activity-registry lookups | 4,096 | 0 |
| Hash activity-registry lookups | 0 | 4,096 |
| Snapshot ordering-policy changes | 0 | 0 |
| Allocations on descriptor hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_ACTIVITY_REGISTRY_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bu_activity_registry_hash_index_preserves_lookup_and_order` covers
  view/window lookup, deterministic snapshots, and duplicate rejection.
- `optimization_batch_20260826bu_activity_registry_hash_index_sorts_snapshots_explicitly` locks
  hash ownership and the explicit deterministic snapshot sort.
- `optimization_batch_20260826bu_activity_registry_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges activity
descriptor registration and lookup.
