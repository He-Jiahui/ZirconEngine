---
title: Runtime09B Instance Upload Hash Membership
category: zircon_runtime
report_id: Runtime09B-instance-upload-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Instance Upload Hash Membership

## Scope

This slice reduces membership cost while deriving static, dynamic, and dirty dynamic instance
upload keys from a visibility BVH update plan. It preserves BVH instance input order in all
published vectors and does not change BVH strategy selection, visibility extraction, or GPU upload
architecture.

## Change

- Build dynamic stable-key membership in `HashSet<u64>` instead of `BTreeSet<u64>`.
- Infer the incremental dirty-key set as the same hash-set type.
- Keep static and dynamic vectors in original BVH instance order.
- Filter the dirty dynamic vector by the original dynamic vector, not by hash iteration order.

## Deterministic Performance Evidence

| Representative 8,192 dynamic keys / 65,536 lookups | Before | After |
|---|---:|---:|
| Dynamic-key membership | ordered O(log n) | average O(1) hash |
| Dirty-key membership | ordered O(log n) | average O(1) hash |
| Published dynamic order | BVH input order | unchanged |
| Published dirty order | dynamic input order | unchanged |

The ignored release gate alternates 17 ordered-membership and hash-membership samples and emits
`RUNTIME09B_INSTANCE_UPLOAD_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires hash-membership P95 to
be at most 60% of ordered-membership P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826v_runtime09b_hash_upload_membership_preserves_input_order`
  exercises the product planner with mixed static/dynamic instances and incremental dirty keys.
- `optimization_batch_20260826v_runtime09b_instance_upload_uses_hash_membership` requires both
  membership queries and rejects a production tree set.
- `optimization_batch_20260826v_runtime09b_instance_upload_hash_membership_performance_evidence`
  verifies equal lookup results, emits both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime09B still needs complete instance/particle/geometry upload budgets, GPU-visible generation
and residency ownership, dirty-range coalescing, device-loss recovery, and product frame/VRAM
qualification across representative scenes.
