---
title: Runtime74 Compile Cache Eviction Key Capacity
category: zircon_runtime
report_id: Runtime797-compile-cache-eviction-key-capacity-2026-09-18
date: 2026-09-18
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime797 Compile Cache Eviction Key Capacity

`UiAssetCompileCache::evict_assets` already uses borrowed hash membership. This
follow-up reserves each temporary removal-key vector from the number of unique
requested asset IDs before filtering the ordered cache and snapshot maps. The
bound is input-sized rather than cache-sized, so a sparse eviction cannot
preallocate the entire cache; extra compile variants still grow normally when
they exceed the lower bound.

## Invariants

- Cache and invalidation-snapshot `BTreeMap` ownership and ordering remain unchanged.
- Duplicate, missing, retained, and multi-variant asset eviction semantics remain unchanged.
- The empty-input early return remains before any temporary allocation.
- No tooling production code is involved.

## Evidence

- RED source contract fails until both `Vec::with_capacity(asset_ids.len())`
  admissions and the lower test module are present.
- GREEN source contract passes `3/3`.
- Lower Rust test `runtime797_compile_cache_eviction_key_capacity_is_input_bounded`
  guards the two bounded collectors.
- Ignored marker `RUNTIME797_COMPILE_CACHE_EVICTION_KEY_CAPACITY_BENCH_V1`
  models 65,536 requested IDs and removes geometric growth events from the
  common one-key-per-request shape (`15 -> 0`).
- The combined current Runtime/Editor source-contract loader covers 561 files
  and passes 2,002/2,002 tests in 12.503s; this is source/model evidence only.
- Exact-file Rustfmt passes; managed Cargo/Release allocation and product
  hot-reload p50/p95/p99 evidence remain pending.

## Remaining work

Run Runtime797 with Runtime795/796 and the current Editor batch in one managed
Windows invocation once the external worktree admission boundary is clean.
Do not promote this local capacity model to product acceptance.
