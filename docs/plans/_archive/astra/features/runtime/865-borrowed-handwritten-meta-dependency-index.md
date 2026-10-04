---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/643/2026-09-21-borrowed-handwritten-meta-dependency-index.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/866-resolved-dependency-output-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs
tests:
  - zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution/optimization_batch_jd_runtime643_tests.rs
  - tools/tests/test_runtime865_handwritten_meta_dependency_borrowed_index_performance_contract.py
---

# Runtime865 Borrowed Handwritten Metadata Dependency Index

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime643 restored metadata/root dependency merge | Borrow both membership indexes, classify incoming dependencies once into a compact dual-target flag buffer, reserve exact output counts, move single-owner values, and clone only for true dual ownership. Existing-first and first-seen order remain unchanged. | Intentional RED `4/4` → GREEN `4/4`; lower divergent-meta/root parity model and ignored `RUNTIME865_HANDWRITTEN_META_DEPENDENCY_BORROWED_INDEX_BENCH_V1` marker are wired. The 4,096+4,096+4,096 model changes URI clones `20,480→4,096` (`80%` deterministic reduction); the related Runtime/asset batch passes `40/40`. Managed Cargo/Release, allocator, and asset-restoration product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

The post-change non-Tooling loader passes `4216/4216` tests across `993` files
in `458.125s`, and `100/100` hashes match across the `19` current dated
Runtime/Editor optimize records. These do not replace managed gates.

## Scope boundary

This slice changes only metadata/root dependency merge scratch ownership. It
does not change extraction, URI normalization, artifact restoration, registry
generation authority, reload publication, or tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs` | `A1207221C7FB47BA6F8595843484D73CD723C3CC7876C3449537213FF3D03A73` (shared current-worktree hash after Runtime866) |
| `zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution/optimization_batch_jd_runtime643_tests.rs` | `91995ECB1AFD392925A6A73E1D0390EDA0B409901AAECB36730797E659227D93` |
| `tools/tests/test_runtime865_handwritten_meta_dependency_borrowed_index_performance_contract.py` | `B1790BC412C28602CD7BB6C060056DF5C30D7DA09468C75FF5C85474D97DEC07` |

## Managed gate

Runtime865 was submitted with Runtime864 in the asynchronous v7 current-source
lane and was not submitted individually. Keep this entry pending until Windows
compilation, lower/ignored Release execution, allocator evidence, and asset-
restoration product p50/p95/p99 results are available.
