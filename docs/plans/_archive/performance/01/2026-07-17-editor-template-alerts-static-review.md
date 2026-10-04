---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alert_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alert_glyphs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - alert identity/style/inline/toast/glyph geometry/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - 1/100/10000 alert identity/theme/text/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template alerts逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_alert_glyphs*`、`template_alerts*`与tests共 **29/29** 个Rust文件、**1,184** 行已逐文件阅读。覆盖inline alert、toast、tone/identity、style、layout、status/close/warning glyph commands及identity/style/geometry/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，规模trace尚未完成，因此本组仍留在`pending.md`。

## P0：paint频率身份字符串和命令放大

`WorkbenchToastRoot`每次paint先由`is_standalone_toast`复制label并分配lowercase String，随后text command再次复制label。未命中standalone条件时，`alert_tone`把control/icon/validation/text-tone/variant/label六段`format!`成一段String，再整体lowercase；通用Alert也走同一路径。宽Toast还逐paint把固定`"UNDO"`转成owned String。以上工作都只随node generation变化，却由stable paint重复承担。

手工glyph成本有界但会放大最终command：warning mark发出6个triangle row quad和2个mark segment，close mark发出8个quad；toast同时包含surface、success glyph、text、action和close glyph。PERF-MVP-178的compiled segment能消除重建，却不能自动减少最终primitive/draw数量，因此需与PERF-MVP-179的asset/mask路径一起用产品counter决定是否收敛为单资源命令。

PERF-MVP-199要求presentation/template projection只在changed node构建typed `AlertPaintSpec`，一次解析kind/tone/state/theme、共享label与固定action text，并缓存geometry和ordered commands。stable generation不得再执行label copy/lowercase/format、selector/theme读取或command build。Slint的dependency-tracked item cache作为generation失效与稳定item跳过工作的参考边界。

## 动态验收

在1/100/10,000 inline alerts/toasts及info/success/warning/error、normal/hover/focus/press/disabled状态上记录identity probe、format/lowercase/owned text bytes、selector/theme acquisition、glyph/text/total commands与draw。changed alert identity/tone/style/theme/label各<=1且临时identity String=0；stable generation以上计数及command build=0。shipped MVP alert glyph若有asset则fallback=0；保留手工fallback时每glyph compiled/RHI command应收敛为1或提供产品trace证明多quad预算可接受，并保持tone/state/action/clip/order/text与GPU/Softbuffer pixels parity。
