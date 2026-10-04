---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_text
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - text-input focus/button/property label tests
  - default font metric projection test
  - current-source Windows Cargo pending
  - 1/100/10000 label allocation/theme-lock/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template node label/text逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_node_labels.rs`、`template_node_labels/**`、`template_node_labels_tests/**`、`template_node_text.rs`与`template_node_text/**`共 **14/14** 个Rust文件、**383** 行已逐文件阅读。覆盖focus/input/property/fallback label、text eligibility/geometry/metrics/command与行为tests。当前源Cargo与规模trace未完成，因此仍留在`pending.md`。

## 判定与热点

Focus匹配、input-kind、eligibility和geometry均为有界match/算术，没有I/O、队列或无界算法。热点是重复owned projection：`template_node_label`无论来源是focus、property、text/value/options均返回String；`push_template_text_fallback_command`先建最终label，`text_rect_for_node -> is_leading_icon_text_node`又用`template_node_label(node,None)`只检查非空，icon+text image geometry还会第三次调用。Property label会`format!`，普通label也`to_string()`，因此同node可在一帧复制/格式化2–3次后只保留最终command的一份。

`node_font_size`还为每node读取全局metrics，归PERF-MVP-161。Text layout/raster归PERF-MVP-156，String深copy归PERF-MVP-174，稳定generation重复label/command build归PERF-MVP-178。局部可让一次resolved label/has-label沿image/text geometry借用；不得建立第二套label cache或改变focus/property/input优先级。

## 动态验收

在1/100/10,000普通、icon+text、input-focus、property nodes上记录label builder calls、formatted/copied bytes、identity probes、theme lock、text-command build与layout/raster scope。局部gate要求每changed node label resolve≤1；最终stable generation这些计数为0且frame theme lock不随node数增长。保持button忽略focus值、input消费focus值、property label/value顺序、leading icon/text geometry、font clamp、clip/z/opacity与pixels一致。
