---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_axis_values.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows_tests/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_row_metrics.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - property axis parsing/component identity/layout/style tests
  - current-source Windows Cargo running
  - 1/100/10000 property-row parse/allocation/theme/metrics/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template property rows逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_property_axis_values.rs`、`template_property_rows*`、tests与共享`template_row_metrics.rs`共 **21/21** 个Rust文件、**857** 行已逐文件阅读。覆盖property/component identity、axis/scalar解析、layout、field/text commands、theme-derived row metrics/palette及现有axis/component tests。当前源Cargo、规模计数与产品像素trace未完成，因此仍留在`pending.md`。

## P0：属性值每paint重新解析、分配并重复读取theme

`property_axis_values`把值按空白切分，给每个axis分配String、给每个value token分配String，再`join`为新的value String并写入动态Vec。典型`X 0 deg Y 90 deg Z -12.5 deg`在command build前就产生output/current Vec、3个axis、6个token与3个join结果；随后label、3个axis和3个value又各`to_string()`复制进HostPaintCommand，单行超过20次heap allocation/reallocation是可达的。

同一三轴row在label/value layout、axis label/field/text与surface helper中反复调用`workbench_row_metrics()`，静态调用展开约 **28次metrics读取**；row入口与每个axis surface合计约 **4次palette读取**。每次读取还重新派生完整`WorkbenchRowMetrics`或`WorkbenchRowPalette`，即使同一frame/theme generation完全未变。

PERF-MVP-194要求property/template projection直接提交typed scalar/axis values和shared text，不在paint解析展示字符串。Changed row compile构建一次`PropertyRowPaintSpec`，只借用一次统一theme metrics/palette snapshot并预解geometry/style/text；stable generation由PERF-MVP-178复用compiled segment。Parser仍可作为输入兼容边界，但不得留在每帧paint owner，也不得用不受presentation generation约束的字符串缓存掩盖问题。

## 动态验收

在1/100/10,000 scalar、2/3/4-axis、带单位与focused/pressed/selected property rows上记录parser calls、Vec/String allocation/bytes、theme/metrics acquisition/derivation、text copies与commands。Changed row要求parse=0、owned value copy=0、theme/metrics各<=1；stable generation以上计数及command build均为0。保持axis grouping/unit、component label width、field border/focus、layout/order/text与GPU/Softbuffer pixels parity。
