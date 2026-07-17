---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_row_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_row_geometry/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_row_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_row_glyphs/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - tree identity/style/depth/guide/disclosure/object/action asset/glyph/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - visible-range and 1/100/10000 row-depth selector/resource/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template tree rows逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_tree_row_geometry*`、`template_tree_row_glyphs*`、`template_tree_rows*`与tests共 **32/32** 个Rust文件、**1,579** 行已逐文件阅读。覆盖tree identity/state/style、depth/indent guides、disclosure/object/label/actions geometry、asset/fallback glyph及identity/style/depth/action asset/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，visible-range与规模trace尚未完成，因此仍留在`pending.md`。

## P0：visible rows乘depth放大的theme/metrics与资源解析

一行可为surface三项、disclosure、object color/state、label和两个action color执行约9次完整tree-row selector。基础几何、label和两枚action约15–17次metrics/palette投影；每个indent guide又在`tree_guide_rect`及其`tree_guide_x`中重复获取metrics，额外约2次/depth。深层层级因此把本可一次共享的theme work放大为`visible rows * depth`。

常规行还会分别解析disclosure、object、eye和secondary action四个资源并复制label。现有测试要求四类shell资产都真实命中且无missing-icon，因此手工disclosure/action/object的3–7 quad fallback应是零命中异常路径，而不是常规paint成本。

PERF-MVP-204要求changed row一次构建typed `TreeRowPaintSpec`，包含identity/icon/action kinds、state、单一theme/metrics/style snapshot、shared label、depth guides geometry和四个resource handles。PERF-MVP-177提供展开树的shared visible range，PERF-MVP-178保留compiled segment，PERF-MVP-181拥有resource generation。单节点展开/选中只patch受影响行/后代可见区，不得重建完整层级。

## 动态验收

在1/100/10,000 rows、depth 0/1/8/64、visible+overscan/offscreen及collapsed/expanded/selected/checked/loading/disabled状态记录visited rows、guide visits、selector/theme/metrics acquisition、label bytes、resource resolve/fallback、static/dynamic build与commands。changed visible row各projection<=1且guide metrics acquisition不随depth增长；stable visible以上及build=0；offscreen=0；shipped four-glyph fallback=0。保持depth/indent override、guide/disclosure/object/action identity、state priority、clip/order和GPU/Softbuffer pixels parity。
