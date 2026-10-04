---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/04/2026-09-13-catalog-projection-input-capacity.md
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/record_projection.rs
tests:
  - tools/tests/test_editor_catalog_projection_input_capacity_performance_contract.py
---

# Editor Catalog Projection Input Capacity

Editor 的 Runtime catalog 全量投影现在按 authoritative Runtime asset count
预留 `catalog_by_uuid` 与 `uuid_by_locator` 两张 HashMap，并按 metadata
diagnostics 与 reference repairs 的已知数量预留诊断输出。投影仍由同一条
Runtime catalog-input generation 单遍驱动，不引入第二份 authority 或额外
计数扫描。

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor04 / catalog input projection | Reserve map and diagnostic output capacity on full refresh | implemented_pending_validation | TDD RED/GREEN source contract `3/3`; the comprehensive non-tooling Runtime/Editor performance/pressure batch covers 548 files and passes `2039/2039` in `22.986s`; managed Cargo, Release allocation, and catalog p50/p95/p99 evidence remain pending. |

## 性能边界

对 `N` 个 Runtime asset，投影仍为 `O(N)`；两张 map 不再因逐条 insert
发生几何 rehash，诊断 Vec 也不再因 repair append 触发常规扩容。排序、
artifact/reference 语义和 preview currentness 均未改变。

## 受管验证

该切片加入现有 Runtime/Editor 合批，不单独启动 Cargo；当前仍为
`implemented_pending_validation`，且不改变外部 `E:\Git\zr_vm` dirty/overlay
门禁导致的受管验证待定状态。
