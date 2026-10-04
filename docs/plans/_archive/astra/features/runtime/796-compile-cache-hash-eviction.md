---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/74/2026-08-26-compile-cache-borrowed-hash-eviction.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
tests:
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache/hash_eviction_tests.rs
  - tools/tests/test_runtime_compile_cache_hash_eviction_performance_contract.py
---

# Runtime796 · compile-cache borrowed hash eviction

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime74 compile cache | Build temporary eviction membership as `HashSet<&str>` while retaining ordered cache/snapshot maps, preserving duplicate collapse, removal counts, and cache authority. | TDD source contract `3/3`; lower multi-asset semantics regression; ignored `RUNTIME74_COMPILE_CACHE_HASH_EVICTION_BENCH_V1` now samples 101 alternating runs and reports `sample_count=101`; 65,536-admission model removes transient ID clones and changes membership from ordered `O(A log U)` to expected `O(A)`; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

## Implementation evidence

`UiAssetCompileCache::evict_assets` now borrows caller IDs into a temporary
hash membership set and probes compiled/snapshot IDs by borrowed views. The
ordered `BTreeMap` stores and key-removal sequence remain unchanged, so cache
ordering and invalidation report semantics are preserved.

## Source fingerprints

The shared compile-cache source fingerprint is refreshed after the adjacent
Runtime797 capacity follow-up; the Runtime796 hash-membership assertions remain
unchanged.

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs` | `A3C55D4B4B02B46569A514ADF2DE8E8CFC72964683970D962215111D2C17279B` (refreshed after Runtime797) |
| `zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache/hash_eviction_tests.rs` | `DC8D361BB3C78CBBA6D144D19FFCFE87303C1A59E6642FAE7C9ED35A3BE4F845` |
| `tools/tests/test_runtime_compile_cache_hash_eviction_performance_contract.py` | `73BCF0537906FA5EE49B881A3F3558A20351C0596E85CC9C6A95E614E1EC9C21` |

The exact five-file Rustfmt set passes, and the new source contract is included
in the current combined Runtime/Editor batch (`2002/2002` across `561` files in
`12.503s`). Local receipts do not establish managed Cargo compilation, Windows
Release allocation counts, or product hot-reload p50/p95/p99.

## 性能与受管验证边界

The deterministic 65,536-admission model removes transient owned ID strings and
retains the existing two-phase removal semantics. The release marker now uses 101 alternating
samples for a more stable local percentile estimate. Keep this record
`implemented_pending_validation` until the asynchronous managed batch supplies
the ignored Release marker and product gates. Tooling production code remains
deferred.
