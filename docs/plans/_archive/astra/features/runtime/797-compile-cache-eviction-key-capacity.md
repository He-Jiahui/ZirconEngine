---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/74/2026-09-18-compile-cache-eviction-key-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/795-ui-resource-reference-streaming-visitor.md
  - docs/plans/astra/features/runtime/796-compile-cache-hash-eviction.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
tests:
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache/eviction_capacity_tests.rs
  - tools/tests/test_runtime_compile_cache_eviction_key_capacity_performance_contract.py
---

# Runtime797 · compile-cache eviction-key capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime74 compile-cache eviction | Reserve both temporary removal-key vectors from the unique requested asset-ID count, keeping the bound input-sized and preserving ordered map authority. | TDD RED→GREEN source contract `3/3`; lower two-collector regression; ignored `RUNTIME797_COMPILE_CACHE_EVICTION_KEY_CAPACITY_BENCH_V1`; deterministic 65,536-ID model `15→0`; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs` | `A3C55D4B4B02B46569A514ADF2DE8E8CFC72964683970D962215111D2C17279B` |
| `zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache/eviction_capacity_tests.rs` | `CBDD0CB61B9B1D6CFB267100783E14922EA951A4CF7BD9BEE1877FA293E8E22E` |
| `tools/tests/test_runtime_compile_cache_eviction_key_capacity_performance_contract.py` | `457339509F25C16D9F189164B09547700482A079C98E44DD580B7C10FAB22D17` |

The lower module is mounted next to the existing borrowed-hash eviction
regression. Local Rustfmt and source-contract checks do not establish Cargo
compilation, Release allocation counts, or product hot-reload p50/p95/p99.

## 性能与受管验证边界

The input-sized reservation removes geometric growth in the modeled
one-key-per-request workload without reserving the whole cache for sparse
evictions. Keep this record `implemented_pending_validation` until the
asynchronous managed batch supplies the lower Rust and product gates. Tooling
production code remains intentionally deferred.
