---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - status identity/style/metrics/chip/signal/icon/glyph geometry/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - idle and 1/100/10000 status text/theme/metrics/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template status controls逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_status_control_geometry*`、`template_status_controls*`、`template_status_glyphs*`与tests共 **34/34** 个Rust文件、**1,806** 行已逐文件阅读。覆盖status signal/chip/icon identity、state/style、host theme projection、metrics/geometry、label/value split与measurement、manual glyph commands及identity/style/metrics/geometry/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，idle/规模trace尚未完成，因此仍留在`pending.md`。

## P0：常驻状态栏的文本、metrics与glyph重复工作

Status chip的`label:value`路径先由`template_node_label`复制完整标签，再用`format!`和`to_string`分配最终两个text command字符串，并在每次paint重新测量value。Surface与两段text geometry合计约10次完整status metrics投影。Signal约6次metrics投影并复制label；icon button约3次。各selector已正确只取一次palette，但stable status bar仍重复上述projection与command build。

手工status glyph的最终primitive成本也明显：Snap为5个quad，World为outline加4段，Target为outline、中心和4段；可见surface会再增加一个command。PERF-MVP-178只能消除stable build，不能消除最终draw放大，因此仍需PERF-MVP-179的产品asset/fallback counter决定单mask/atlas收敛。

PERF-MVP-201要求changed node一次构建typed `StatusControlPaintSpec`，包含kind/state、单一theme/metrics snapshot、shared或精确两段text、缓存measurement/geometry及ordered commands。稳定status generation不得再split/format/copy/measure、读取theme/metrics或重建command；仅实际数值/状态变化patch对应动态段。

## 动态验收

记录30秒idle及1/100/10,000 status controls的identity、label/split/format bytes、text measurement、palette/metrics acquisition、static/dynamic segment build、glyph/total commands与draw。changed control identity/style/theme/metrics各<=1，文本owned bytes只等于最终command必要字节；stable generation以上及command build=0，value-only变化不重建surface/glyph。shipped glyph fallback=0或每fallback compiled/RHI command=1；保持state priority、label/value colors/alignment、signal semantics、clip/order和GPU/Softbuffer pixels parity。
