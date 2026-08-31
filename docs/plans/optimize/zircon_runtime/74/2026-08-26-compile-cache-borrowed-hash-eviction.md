---
title: Runtime74 Compile Cache Borrowed Hash Eviction
category: zircon_runtime
report_id: Runtime74-compile-cache-borrowed-hash-eviction-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime74 Compile Cache Borrowed Hash Eviction

## Scope

This slice optimizes multi-asset UI compile-cache eviction. The old path cloned every requested
asset ID into an owned `String` and used an ordered set even though eviction only needs membership.
Cache-entry ordering, snapshot-slot ownership, removal counts, and hot-reload publication remain
unchanged.

## Change

- Collect the caller-owned `&str` IDs directly into a `HashSet<&str>`.
- Probe compiled document IDs through `as_str()` and snapshot slots through their existing borrowed
  asset-ID view.
- Preserve duplicate-input collapse and the existing two-phase key collection/removal sequence.
- Leave the ordered compile-cache and invalidation-snapshot maps unchanged.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique assets / 16,384 probes | Before | After |
|---|---:|---:|
| Membership construction | `O(A log U)` ordered insert | expected `O(A)` hash insert |
| Membership probe | `O(log U)` | average `O(1)` |
| Transient owned asset-ID strings | 65,536 | 0 |
| Cache and snapshot removal semantics | matching IDs removed | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME74_COMPILE_CACHE_HASH_EVICTION_BENCH_V1`. Acceptance requires borrowed hash-membership P95
to be at most 60% of owned ordered-membership P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ah_runtime74_hash_eviction_preserves_multi_asset_semantics` compiles
  three real UI documents and verifies duplicate, missing, retained, and evicted asset behavior.
- `optimization_batch_20260826ah_runtime74_compile_cache_uses_borrowed_hash_eviction` requires the
  borrowed hash boundary and rejects the old string-cloning ordered set.
- `optimization_batch_20260826ah_runtime74_compile_cache_hash_eviction_performance_evidence`
  checks output equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts passed before managed
  validation submission.

## Remaining Parent-plan Work

Runtime74 still needs terminal product-scale hot-reload qualification, generation-qualified
compiled endpoints, atomic tree replacement, component-state migration, binding reinstallation,
subscription retirement, and last-good rollback. This slice only changes the temporary membership
index used by compile-cache eviction.
