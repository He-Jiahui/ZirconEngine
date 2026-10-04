---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/589/2026-09-20-v2-file-source-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/file_source_capacity_tests.rs
tests:
  - tools/tests/test_runtime_v2_file_source_capacity_performance_contract.py
---

# Runtime855 · V2 file source capacity

## 完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime V2 file-source BFS | Reserve the known `paths.len()` root-input bound for both the import queue and loaded-source output, preserving canonical de-duplication, transitive import order, and empty-input behavior. | TDD source/model contract `4/4` after a RED run with two structural failures; lower queue/source-order regression and ignored `RUNTIME855_V2_FILE_SOURCE_CAPACITY_BENCH_V1` marker are wired. The 4,096-unique-root no-import model changes queue and source growth events `11→0`. The batched non-tooling performance loader passes `2420/2420` across `658` files in `33.490s`; the current expanded source-contract loader passes `4072/4072` across `962` files in `139.499s` (performance-or-contract filename filter, tooling/export/coordinator excluded); managed Cargo/Release, allocator, and product file-source p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only initial vector capacity in the Runtime V2 file-cache
source collector. It does not alter path canonicalization, import resolution,
asset-ID indexing, source ordering, cache keys, or tooling production. The
shared `file_cache.rs` owner also carries Runtime589 source-move changes, which
remain attributed to their own record.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/file_cache.rs` | `B3D29685042A522E0C7D30374BD5708D834F895CC146DBC1F885D2A6E981A513` |
| `zircon_runtime/src/ui/v2/file_source_capacity_tests.rs` | `B689AB0238FF3C23B184C2E25B8650359165277C64D64A95506BD4EB90B6CEEE` |
| `tools/tests/test_runtime_v2_file_source_capacity_performance_contract.py` | `81B6CA902D5D785F614C255300ADEFCD36817929B9A029368737D184DF9E44EF` |

## Managed gate

No managed Windows Cargo/Release validation command is started locally and
coordinator status is not polled. Keep this entry `implemented_pending_validation` until the combined
owner-attributed Windows Release lane proves current-source compilation,
lower-test reachability, allocator behavior, and file-source product
p50/p95/p99 evidence.
