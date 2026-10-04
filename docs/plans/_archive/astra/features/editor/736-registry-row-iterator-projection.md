---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/04/2026-09-13-registry-row-iterator-projection.md
related_code:
  - zircon_editor/src/core/asset/index.rs
tests:
  - tools/tests/test_runtime_editor_registry_iterator_projection_performance_contract.py
---

# Editor Registry Row Iterator Projection

Editor 资产行投影现在从 Runtime registry 的 exact-size `entries_iter()`
直接填充预留的结果 Vec，避免先由 `entries()` 物化第二个 entry Vec。行顺序、
metadata、dirty/importing 状态和 Runtime authority 不变。

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor04 / asset row projection | Stream canonical registry entries into one reserved row output | implemented_pending_validation | TDD RED/GREEN source contract `2/2`; combined Runtime/Editor asset batch `620/620`, focused slice `37/37`, broader performance/pressure discovery `1358/1358`; scoped structural checks pass; managed Editor row/catalog p50/p95/p99 evidence remains pending. |

## 性能边界

行投影仍为 `O(N)`，但只保留一个有界结果分配，删除中间 entry-pointer
向量；这不宣称增量 catalog 或产品 Asset Browser 延迟门禁已完成。

## 受管验证

该切片加入现有 Runtime/Editor 合批，不单独启动 Cargo；当前仍为
`implemented_pending_validation`。
