---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: weight-heatmap-unbounded-cell-source-product
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/07
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_weight_heatmap.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_weight_heatmap/field.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_weight_heatmap/markers.rs
---

# Weight heatmap unbounded cell-source product

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`template_weight_heatmap*` 10/10 个 Rust 文件
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：performance会话直接修复source projection与grid admission；heat generation、worker调度和source invalidation属于domain editor长期所有权。

## 失败现象与复现证据

Authored columns/rows仅做最小值1，没有上限。每个cell重新遍历sources、调用row_data clone并计算`exp`，形成C×R×S主线程CPU/clone；每cell发一条quad，commands为C×R。Markers再遍历sources并按scanline扩命令，stable frame全量重算。

## 最低共享层根因

Domain projection没有发布bounded immutable heat generation或shared source slice；painter直接拥有采样分辨率、数值求值、source model访问和命令展开。

## 架构修复验收

- 直接修复后每generation source row_data≤S，grid cells受plot pixels与hard budget约束且全部sources仍参与max influence。
- Editor07发布heat generation并把高source CPU求值调度到worker，source change精确失效。
- Render13以retained texture/compute result和bounded marker batch表达，EditorUI08只提交handle与dynamic selection。
- 1/16/256 dimensions×1/100/10,000 sources报告exp/CPU p95/alloc/commands/uploads；stable compute/upload=0。
- 保持gradient、legend、max influence、markers、clip与Softbuffer/RenderDoc pixels等价。

## 禁止临时方案

- 不得静默截断sources或只降低默认rows/columns。
- 不得在painter建立无generation约束的第二份heat cache。
- 不得把同一完整source model继续按cell访问，即使grid有上限。

## 修复结果与回传

Open state: `待performance会话回传single-source-projection与bounded-grid修复；待Editor07/Render13回传worker/texture-compute generation证据`。
