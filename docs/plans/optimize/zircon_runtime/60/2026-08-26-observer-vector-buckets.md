---
title: Runtime60 Observer Vector Buckets
category: zircon_runtime
report_id: Runtime60-observer-vector-buckets-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime60 Observer Vector Buckets

## Scope

This slice replaces the three private ECS observer callback buckets from `Arc<BTreeMap<...>>` to
`Arc<Vec<...>>`. Observer IDs are allocated globally and monotonically, so appending to each bucket
retains registration order without maintaining a tree node for every callback.

Callback snapshots remain immutable through `Arc` clone-on-write. Registering or removing an
observer during dispatch therefore cannot mutate the active snapshot, while vector removal keeps
the relative order of every surviving callback. The existing hash indexes still own event,
component, entity, and observer-location point lookup.

## Performance Workload

The release workload traverses 16,384 registered observer payloads 256 times per sample and
measures only callback-bucket storage traversal, excluding callback body cost.

| Work per dispatch traversal | Before | After |
|---|---:|---:|
| Tree-node iterator steps | 16,384 | 0 |
| Contiguous vector elements | 0 | 16,384 |
| Registration-order sort | 0 | 0 |
| Snapshot clone during dispatch | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME60_OBSERVER_VECTOR_BUCKETS_BENCH_V1`. Acceptance requires vector traversal P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826cg_observer_vector_bucket_preserves_registration_order` covers
  monotonic insertion, middle removal, missing removal, and stable survivor order.
- `optimization_batch_20260826cg_observer_vector_bucket_keeps_snapshot_dispatch_contract` locks
  the three vector bucket owners, clone-on-write mutation helper, and contiguous dispatch loops.
- `optimization_batch_20260826cg_observer_vector_bucket_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime60 still owns checked observer identity exhaustion, owner-scoped retirement, scheduling,
parallel event delivery, and product-scale callback qualification. This slice only converges the
private immutable callback-bucket layout.
