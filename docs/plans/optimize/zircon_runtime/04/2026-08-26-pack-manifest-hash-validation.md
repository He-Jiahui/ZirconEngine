---
title: Runtime04 Pack Manifest Hash Validation
category: zircon_runtime
report_id: Runtime04-pack-manifest-hash-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Pack Manifest Hash Validation

## Scope

This slice removes ordered-tree membership from `.zrpack` asset-path and chunk-table validation.
Manifest asset and chunk ordering remains a separate `windows(2)` contract, and the canonical
chunk-size lookup remains a `BTreeMap`. The optimized sets are private validation state and are not
serialized or iterated into pack output.

## Change

- Validate borrowed asset paths with `HashSet<&str>` while preserving first-duplicate errors.
- Validate borrowed chunk hashes with `HashSet<&[u8; 32]>`.
- Compare referenced and declared owned chunk hashes with `HashSet<[u8; 32]>` equality.
- Preserve sorted-path and sorted-chunk checks after duplicate validation.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique paths | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Validation-key allocations | 0 | 0 |
| First duplicate and sorted errors | input ordered | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME04_PACK_MANIFEST_HASH_VALIDATION_BENCH_V1`. Acceptance requires hash validation P95 to be
at most 60% of ordered validation P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ab_runtime04_hash_manifest_validation_preserves_first_duplicate_error`
  exercises the product validator with a repeated asset path.
- `optimization_batch_20260826ab_runtime04_pack_manifest_uses_hash_membership_and_sorted_windows`
  requires four hash membership indexes while retaining both sorted-window checks.
- `optimization_batch_20260826ab_runtime04_pack_manifest_hash_validation_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime04 still needs range-backed runtime mounts, authoritative cook graph integration, typed
artifact manifests, bounded streaming decompression, signed package admission, and crash-safe
promotion. This slice only improves current pack manifest validation.
