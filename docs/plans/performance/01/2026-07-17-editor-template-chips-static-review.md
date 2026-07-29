---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chip_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chip_glyphs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - chip identity/style/metrics/layer/glyph geometry/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - 1/100/10000 chip theme/metrics/text/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template chips逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_chip_glyphs*`、`template_chips*`与tests共 **18/18** 个Rust文件、**739** 行已逐文件阅读。覆盖chip identity/state/style、host theme projection、metrics/geometry/layers、label、chevron glyph与identity/style/metrics/geometry/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，规模trace尚未完成，因此仍留在`pending.md`。

## P0：单节点theme/metrics重复投影

带chevron的chip每paint分别通过surface、border、text和glyph helper读取约4次palette。Surface读取2次metrics，label路径读取5次，chevron又读取2次，共约9次完整`HostControlMetrics`投影；label始终复制为owned String，`chip_has_chevron`也在label reserve和glyph dispatch各探测一次。每项算术有界，但全都位于per-node stable paint，规模按可见chip数线性放大。

PERF-MVP-200先在入口一次构建`ChipPaintSpec`，包含typed identity/has-chevron/state、单一palette/metrics snapshot、shared label、resolved colors/geometry和ordered commands，surface/text/glyph只借用spec。PERF-MVP-178最终按node generation保留compiled segment。当前3段chevron fallback仍需由PERF-MVP-179的产品asset/fallback counter决定是否收敛成一个mask/atlas command。

## 动态验收

在1/100/10,000 chips、带/不带options/chevron及normal/hover/focus/press/open/selected/disabled状态记录identity/chevron probes、label bytes、palette/metrics acquisition、command build与glyph/total commands。changed chip identity/has-chevron/label/palette/metrics各<=1；stable generation以上及command build=0。shipped MVP chevron若有asset则fallback=0；保留fallback时每glyph compiled/RHI command=1或以产品trace证明3-quad预算可接受。保持state priority、declared metrics、label reserve、clip/order和GPU/Softbuffer pixels parity。
