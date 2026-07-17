---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_control_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_control_geometry/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - segmented identity/options/metrics/geometry/style/tab/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - option clone/lowercase direct fix pending
  - 1/100/10000 segmented option/style/theme/metrics/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template segmented controls逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_segmented_control_geometry*`、`template_segmented_controls*`与tests共 **28/28** 个Rust文件、**1,208** 行已逐文件阅读。覆盖segmented/tab identity、ModelRc options、selection、metrics/geometry、style、label/divider/underline commands及现有options/style/tab/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，规模trace也未完成，因此仍留在`pending.md`。

## P0：按option放大的clone、selector与metrics projection

`segmented_options`逐row调用owned `row_data`后又执行`option.to_string()`，所以每个option至少复制两次；`selected_segment_value`把整个selected文本lowercase成新String；`segment_label`再为每个展示label分配String。三项都在stable paint每帧执行。

Body入口只计算一次style，但group label再次完整select，selected segment再次select，每个option label又调用一次完整selector；N个options可达N+2次style projection。细粒度geometry helper把同一个16字段metrics projection拆成单字段函数：group label、body、每个divider、selected rect/radius、每个label rect/font/line height都重新获取全局metrics，按option数量线性放大。

PERF-MVP-198要求changed-node入口构建`SegmentedControlPaintSpec`：borrowed/真正shared options、typed selected index、一次resolved style/theme/metrics/text snapshot、预解segment geometry和labels。各helper只借用spec，stable generation由PERF-MVP-178复用compiled segment。局部先删除`row_data`后的第二次String复制，并让selected value保持borrowed `&str`配合`eq_ignore_ascii_case`，不等待架构迁移。

## 动态验收

在1/100/10,000 controls、2/3/10/100 options、selected/hover/focus/press/disabled与tab上记录row clone/String bytes、lowercase、selector/state、theme/metrics acquisition、label builds与commands。局部修复后option额外copy=0、selected lowercase=0；changed control selector/style/theme/metrics各<=1；stable generation以上计数及command build均为0。保持ModelRc option order、case-insensitive selection、label capitalization、declared style、geometry/order与GPU/Softbuffer pixels parity。
