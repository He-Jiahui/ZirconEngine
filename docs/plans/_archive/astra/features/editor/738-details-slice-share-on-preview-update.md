---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/04/2026-09-13-details-slice-share-on-preview-update.md
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/generation.rs
tests:
  - tools/tests/test_editor_catalog_details_cow_capacity_performance_contract.py
---

# Editor Details Slice Sharing on Preview Update

单项及批量 preview/catalog 更新在目标 details 尚未物化时现在直接复用已有的
`details_by_asset_index` `Arc`；只有目标已有 details 时才复制指针 slice 并替换
目标行。Asset row、catalog record、generation identity 与 details 内容语义保持不变。

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor04 / preview row COW | Skip unneeded details-slice copy for unmaterialized single-row and batched updates | implemented_pending_validation | TDD RED/GREEN source contract `4/4`; the comprehensive non-tooling Runtime/Editor performance/pressure batch passes `2039/2039` across 548 files; scoped Rustfmt and diff checks pass; managed Cargo, Release allocation, and catalog-update p50/p95/p99 evidence remain pending. |

## 性能边界

未物化 details 的单项或批量更新在 details 状态上从 `O(N)` copy 降为 `O(1)`；已物化
目标保留原有 copy-on-write 与单行替换。更大范围的 chunked immutable generation
仍属于后续 P2-03 架构工作。

## 受管验证

该切片加入现有 Runtime/Editor 合批，不单独启动 Cargo；当前仍为
`implemented_pending_validation`，不改变外部 dirty/overlay 门禁导致的受管验证待定状态。
