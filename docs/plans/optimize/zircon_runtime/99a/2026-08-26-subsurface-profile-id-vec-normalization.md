---
title: Runtime99A Subsurface Profile ID Mask Normalization
category: zircon_runtime
report_id: Runtime99A-subsurface-profile-id-mask-normalization-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99A Subsurface Profile ID Mask Normalization

## Scope

This slice reduces per-frame CPU and allocation work while gathering the subsurface profile slots
used by visible material instances. It follows Runtime99A's current fixed 16-slot SSS table and P2-5
performance-evidence requirement. It does not claim a content-addressed Diffusion Profile service,
dynamic allocator, material-parent resolution, camera-layer filtering, or a scalable SSS kernel.

## Change

- Replace the temporary used-profile `BTreeSet<u32>` with one `u32` active mask.
- Set one bit per visible subsurface material using the shared `ZR_SSS_MAX_PROFILES` contract.
- Expand bits from slot 0 through slot 15 into the existing final vector, preserving ascending
  unique profile IDs.
- Keep the explicit-profile `BTreeMap` and its explicit-first conflict behavior unchanged.

## Deterministic Performance Evidence

| 32,768 mesh profile references / 16 slots | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-tree insertion attempts | 32,768 | 0 | 100% removed |
| Temporary used-profile tree | up to 16 entries | 0 | removed |
| Dedup state | allocated ordered tree | 4-byte mask | allocation removed |
| Final profile IDs | ascending unique | ascending unique | unchanged |

The ignored release gate alternates 17 ordered-tree and fixed-mask samples. It emits
`RUNTIME99A_SUBSURFACE_PROFILE_MASK_DEDUP_BENCH_V1`; acceptance requires mask P95 to be at most 60%
of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826i_runtime99a_profile_mask_preserves_sorted_unique_slots` covers the
  mask-to-vector order and sparse slot behavior.
- `optimization_batch_20260826i_runtime99a_profile_usage_uses_fixed_capacity_mask` rejects the
  transient tree and requires the shared capacity constant.
- `optimization_batch_20260826i_runtime99a_profile_mask_dedup_performance_evidence` emits both P95
  values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting with child traversal disabled, scoped diff checks, and source
  contracts must pass before managed validation submission.

## Remaining Parent-plan Work

Runtime99A still requires stable Diffusion Profile identity, complete conflict/overflow diagnostics,
material-parent and camera-layer convergence, authored thickness, quality tiers, GPU timings, and
the full 4K multi-view qualification matrix. Expanding beyond 16 slots must hard-cut both the GPU
table and this mask contract together.
