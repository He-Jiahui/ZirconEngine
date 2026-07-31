---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: asset-pane-projector-repeated-model-scans
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/retained_host/primitives.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/mod.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/transform.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
tests:
  - tools/tests/test_editor09_asset_content_generation_projection.py
  - zircon_editor/src/ui/workbench/asset_content_layout/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/transform.rs
---

# Asset pane projector repeated model scans

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`paint_workbench_renderer/{docks/pane.rs,docks/pane/**}` 10/10 Rust文件
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：固定资产内容几何、identity index与visible range应由资产模型generation持有，painter只能执行动态scroll/hover投影。

## 失败现象与复现证据

Activity projector初始化曾两次扫描完整node model；Browser projector与Browser content scrollbar的list模式各自最坏四次扫描，之后command pipeline还会再次遍历。性能计划已把两个projector构造器与Browser scrollbar各收敛为单遍几何摘要，并删除旧search helpers；该止损仍会在每次paint重新扫描稳定模型一次。

## 最低共享层根因

资产模型generation没有同时发布content mode、panel/header/grid/preview固定几何、解析后的node identity、row counts和visible range。Painter只能从通用template node DTO重新推导结构。

## 架构修复验收

- Asset model generation change最多构建一次geometry/identity index；stable paint projector-wide `row_data`/identity parse为0。
- Scroll只更新visible range/translation，hover只更新目标row/card动态段，不重建固定几何或全量commands。
- 1/1k/10k nodes报告row_data、parse、clone bytes、alloc、CPU p95；增长由visible nodes而非total nodes主导。
- Activity/Browser list/thumbnail、header/grid/preview、empty/stale、scroll/clip/hover/hit/Softbuffer pixels等价。

## 禁止临时方案

- 不得在painter新增无generation/容量边界的第二份node cache。
- 不得通过截断nodes或取消不可见项语义来伪造常数时间。
- 不得回退为多个`find_*`从row zero重复扫描通用DTO模型。

## 修复结果与回传

Open state: `2026-07-19 generation-owned geometry/identity/fixed+visible row plan、中立generation input、跨DTO共享元数据、projector/scrollbar zero-stable-scan 与旧identity owner删除已落地；静态合同6/6且production workbench→layouts反向依赖为0。独立review从0/1/0收敛为0/0/0，exact manifest已自洽。待managed Cargo、产品像素等价、1/1k/10k row_data/parse/clone/alloc/CPU p95、failure return与managed commit，因此本记录保持open。`

当前切片记录：[2026-07-19-asset-content-generation-projection.md](2026-07-19-asset-content-generation-projection.md)。

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-07-23 13:17 +08:00 | `OPEN / source_review_zero_validation_pending` | 独立复审 0/1/0：业务源码的 generation metadata、zero-stable-scan、visible-row plan 与旧 identity owner 删除均成立；唯一 Important 为 failure exact manifest 遗漏 `asset_content_layout/mod.rs`、asset-content `mod.rs`、template-node `transform.rs` 与 exact-row transform test。现已补齐并保留 `identity.rs` 作为待提交删除项，增量复审 0/0/0；Cargo、像素/规模/p95、fixed return 仍待。 |
| 2026-07-19 15:02-15:22 +08:00 | `OPEN / source_complete_static_green_validation_pending` | 生成期 typed metadata、中立 generation input、DTO 共享保留、Activity/Browser 零扫描投影、精确可见行与旧 parser 删除已完成，静态 6/6、反向层级依赖 0；managed Cargo、产品等价、规模数据、独立 review 与 fixed return 尚未完成。 |
