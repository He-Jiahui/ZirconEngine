---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/129-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/129/2026-09-13-reference-projection-capacity.md
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/details.rs
tests:
  - tools/tests/test_editor_asset_reference_projection_capacity_performance_contract.py
  - zircon_editor/src/tests/host
---

# Editor Asset Reference Projection Capacity

Editor 资产详情的 direct / reverse reference 投影现在使用已知来源数量
预留输出容量，并复用单次 Runtime referencer UUID 集合。保留稳定排序、
未知目标降级和 catalog registry 权威，不改变搜索/引用结果语义。

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor129 / reference projection | Direct/reverse reference row projection reserves known lower bounds and avoids implicit geometric growth | implemented_pending_validation | TDD RED/GREEN source contract `3/3`; batched Editor asset/hierarchy/reference contracts loaded 18 modules and passed `89/89` in `0.499s`; managed Cargo, Release allocation, and reference-projection latency evidence remain pending. |

## 性能边界

排序仍为 `O(R log R)`，但 `R` 条 direct/reverse row 不再依赖 `Vec` 几何扩容；
reverse projection 也不建立额外 UUID 中间列表。Editor129 的统一 Search provider、
typed query 与 field-path 仍由父计划负责。

## 源码快照

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/details.rs` | `4DA999FE03CBE806ED1625CEF0442CF00F491D5B1D273810D7411084FA286A1E` |
| `tools/tests/test_editor_asset_reference_projection_capacity_performance_contract.py` | `A5047317428150EC3A16CB4D4E5C09098AD6F7999885D3F9B697DE0F2E3B1B81` |

## 受管验证

该切片加入现有 Runtime/Editor 合批，不单独启动 Cargo。此前协调器在
`E:\\Git\\zr_vm` 外部脏工作树与 static overlay ownership 门禁处拒绝，
当前仍为 `implemented_pending_validation`。
