---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_structure.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_axis_values.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdown_metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_segments.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_row_metrics.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - dropdown and row metric projection tests
  - icon segment geometry tests
  - property axis grouping test
  - current-source Windows Cargo pending
  - theme-lock/icon-cache/property allocation counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template paint leaf/metrics逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_viewport_scene_structure.rs`、`template_icon_assets.rs`、`template_property_axis_values.rs`、`template_dropdown_metrics.rs`、`template_icon_button_glyph_segments.rs`与`template_row_metrics.rs`共 **6/6** 个Rust文件、**513** 行已逐文件阅读。当前源Cargo、产品paint trace和规模counter未完成，因此仍留在`pending.md`。

## 判定

Viewport base surface与icon segment只做有界几何/command append，没有独立算法热点。Icon asset路径按目标尺寸取像素，bytes/atlas所有权继续由PERF-MVP-150的resource generation处理。Dropdown与row metrics是纯projection，但每个调用都读取全局metrics/palette；调用点分散在dropdown surface/text/glyph、list/property/tree row中，同一node可能重复获取，归PERF-MVP-161一次frame theme snapshot，不另建局部缓存。

`property_axis_values`是明确的paint-frequency allocation：每次属性行绘制先为axis分配String，再为每个value/unit token分配String，随后`join`生成另一份value String；host text command之后仍拥有最终String。它并入PERF-MVP-178的generation-owned compiled segment，稳定node不得重复parse/alloc。若先做局部修复，应以borrowed slice或有界small representation避免per-token allocation，同时保持当前空白归一化、最多四轴、scalar fallback与像素布局。

## 动态验收

记录1/1,000/10,000 property rows的axis parse calls、token/string allocation bytes、command build，以及dropdown/list/property/tree同帧theme lock acquisitions和icon cache hit/miss/upload。Stable generation的axis parse与command build为0；每frame theme lock acquisition不随node数增长；同resource generation的icon raster/upload各至多一次。保持metric projection、axis/unit text、scalar fallback、glyph geometry、clip/order/opacity与pixels一致。
