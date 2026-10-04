---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltip_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltip_glyphs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - tooltip identity/state/theme/metrics/surface/text/arrow/icon/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - hover burst and 1/100/10000 tooltip theme/text/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template tooltips逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_tooltip_glyphs*`、`template_tooltips*`与tests共 **19/19** 个Rust文件、**728** 行已逐文件阅读。覆盖tooltip identity/style/layout/metrics、surface/shadow/title/body、arrow/info glyph及state/theme/metrics/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，hover burst与规模trace尚未完成，因此仍留在`pending.md`。

## P1：重复metrics与按像素扫描线展开arrow

单个tooltip每paint约7次完整metrics投影，title与body各复制一段String。Arrow用border/fill两枚diamond逐扫描线发出quad：command数量约为`size + max(size-2,2)`，在共享上限内虽有界，但默认已经是十余条，最大尺寸接近两倍size；另有2个surface、2段text与3条info glyph。Tooltip数量少于rows，因此列为P1，但hover burst会频繁创建/销毁同一套commands。

PERF-MVP-207要求changed/visible tooltip一次构建`TooltipPaintSpec`，包含state、单一theme/metrics snapshot、shared title/body、bubble/arrow/icon geometry和ordered commands，stable hover target generation由PERF-MVP-178复用。Arrow应由Render13/图标资源拥有一个bounded mask/atlas variant并以单command绘制；若保留扫描线fallback，必须有明确命中与command预算。

## 动态验收

在1,000次同target pointer move、target切换burst及1/100/10,000 tooltips、默认/最小/最大arrow、normal/pressed/focused/disabled状态记录tooltip generation、theme/metrics acquisition、title/body bytes、surface/text/glyph build、arrow/total commands与draw。stable target全部及build=0；changed tooltip各projection<=1；arrow compiled/RHI command=1或fallback有界且产品命中为0。保持layout override、state/tone、shadow/border、title/body、arrow/icon order和GPU/Softbuffer pixels parity。
