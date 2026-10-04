---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/88/2026-09-21-handwritten-dependency-move-append.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/registry/dependency_extractors/mod.rs
tests:
  - zircon_runtime/src/asset/registry/dependency_extractors/dedup_index_tests.rs
  - tools/tests/test_runtime864_handwritten_dependency_move_append_performance_contract.py
---

# Runtime864 Handwritten Dependency Move Append

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime88 handwritten dependency append | Replace the full `AssetUri` clone buffer with a compact acceptance mask, exact destination reserve, and owned candidate moves while preserving existing-first order and first-seen deduplication. | Intentional RED `4/4` → GREEN `4/4`; lower pointer/order regression and ignored `RUNTIME864_HANDWRITTEN_DEPENDENCY_MOVE_APPEND_BENCH_V1` marker are wired. The 4,096-candidate model changes candidate URI clones `4096→0` and full scratch slots `4096→0`, using a 4,096-bit mask; the related Runtime/asset batch passes `36/36`. Managed Cargo/Release, allocator, and asset-import product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

The post-Runtime865 non-Tooling loader passes `4216/4216` tests across `993`
files in `458.125s`, and the current dated-record audit matches `100/100`
source hashes. These are local source/model and integrity receipts only.

## Scope boundary

This slice changes only temporary dependency-append ownership. It does not
change typed extraction, URI normalization, dependency resolution, registry
generation authority, reload publication, or tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/registry/dependency_extractors/mod.rs` | `910C3824F1F1B8B4F88573F20408F4C11C911E1B3B735B27F629BB87AF535A2C` |
| `zircon_runtime/src/asset/registry/dependency_extractors/dedup_index_tests.rs` | `4B88EB371F67E1C7E773B414D39ED3C86CFCA44C8790930B3CE051F19573B4F6` |
| `tools/tests/test_runtime864_handwritten_dependency_move_append_performance_contract.py` | `FE5F86C43EB46A7C4973699D0A45EA1315BCCB90FC8781822B6059C21ED3F19B` |

## Managed gate

Runtime864 was submitted with Runtime865 in the asynchronous v7 batch and was
not submitted alone. Keep this entry pending until that current-source lane
provides Windows compilation, the ignored Release marker, allocator evidence,
and asset-import product p50/p95/p99 results.
