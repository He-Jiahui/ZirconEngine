---
title: Runtime37 Camera Target Hash Counts
category: zircon_runtime
report_id: Runtime37-camera-target-hash-counts-2026-08-26
date: 2026-08-26
session_id: root-runtime37-two-task-camera-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime37 Camera Target Hash Counts

## Scope

This slice replaces the private target/HDR camera-count `BTreeMap` with `HashMap`. Camera output is
already deterministically sorted by render order, target key, and entity before this index is
created. The index is used only to assign each camera's ordinal within its target/HDR stream.

Ambiguity collection remains a `BTreeSet`, so diagnostic order is unchanged. Camera filtering,
descriptor payloads, public camera order, and target-local ordinals retain their existing behavior;
no hash iteration reaches the report.

## Performance Workload

The release workload assigns 65,536 camera entries across 4,096 target/HDR keys.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered target-count probes | 65,536 | 0 |
| Hash target-count probes | 0 | 65,536 |
| Camera sort comparisons | unchanged | unchanged |
| Ordered ambiguity projections | unchanged | unchanged |

The ignored release gate runs 21 alternating sample pairs and emits
`RUNTIME37_CAMERA_TARGET_HASH_COUNTS_BENCH_V1`. Acceptance requires hash count assignment P95 to
be at least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `runtime37_batch_camera_hash_counts_preserve_order_and_indices` covers
  deterministic output order, per-target/HDR ordinals, and ambiguity order.
- `runtime37_batch_camera_target_counts_are_hash_private` locks the private
  hash owner while retaining ordered ambiguity collection.
- `runtime37_batch_camera_target_hash_counts_release_benchmark` reports
  paired release P50/P95 samples and enforces the 30% P95 reduction gate.
- The managed `runtime37_batch_` release gate covers this task and camera-stack entity indexing in
  one Cargo invocation: 2 source contracts, 5 Rust tests, and 2 performance rows. Dynamic marker
  values, integration commit, and WeCom delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime37 still owns camera endpoint authority, director and rig evaluation, view ownership,
history lifecycle, multi-view routing, and product-scale qualification. This slice only converges
the post-sort target/HDR ordinal index.
