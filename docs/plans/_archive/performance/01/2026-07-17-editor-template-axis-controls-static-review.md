---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_field_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_field_style
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - axis identity/metrics/palette/geometry/style/command/pixel tests
  - current-source Windows Cargo running
  - 1/100/10000 transform-axis theme/metrics/text/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template axis controls逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_axis_labels*`、`template_axis_value_field_style*`、`template_axis_value_fields*`及对应tests共 **43/43** 个Rust文件、**1,684** 行已逐文件阅读。覆盖Transform X/Y/Z与scale-link identity、theme-derived metrics/palette、field surface/text、geometry/layering以及完整identity/style/pixel tests。当前源Cargo、规模计数与产品像素trace未完成，因此仍留在`pending.md`。

## P0：Transform字段每paint重复theme/metrics projection

一个axis value field先在`axis_field_rect`读取并派生一次metrics，surface为radius再读取一次，text为inset/font/line height第三次读取。Surface又分别通过background和border读取全局palette，text color第三次读取；相同node/state因此每paint有 **3次metrics + 3次palette** 全局读取与重复projection。值文本最后仍`to_string()`复制进command。

Axis label的text/scale-link路径各读取一次metrics和一次palette，但palette projection每节点重新对5个颜色执行RGB缩放、round与clamp；同一theme generation的派生结果没有复用。Scale-link稳定为3个quad；只有产品trace证明draw/command放大后才按PERF-MVP-186改为typed glyph，不能为了一个固定控件建立独立资源authority。

PERF-MVP-192要求changed-node compile构建一次`AxisControlPaintSpec`：typed axis/scale-link/value-field identity、一次theme/metrics snapshot、resolved palette/style/geometry和borrowed/shared value text。Surface/text/scale-link只借用spec；stable generation由PERF-MVP-178直接复用compiled segment。Theme-derived axis palette应随统一theme generation派生一次，而不是在每个node内重算。

## 动态验收

在1/100/10,000 Transform position/rotation/scale labels与value fields、normal/hover/selected/focused/pressed/disabled/error/warning状态上记录identity、theme/metrics acquisition、palette derivation、value String bytes、text measurements与commands。Changed node要求theme/metrics各<=1、palette derivation<=1/theme generation、value owned copy=0；stable generation以上计数及command build均为0。保持现有43文件覆盖的axis identity、scale-link geometry/order、declared/disabled colors、field height/state/border width/text与GPU/Softbuffer pixels parity。
