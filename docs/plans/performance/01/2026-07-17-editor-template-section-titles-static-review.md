---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_title_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_title_glyphs/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - section title identity/theme/metrics/icon/strong-text/asset-contract/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - 1/100/10000 title theme/metrics/text/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template section titles逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_section_title_glyphs*`、`template_section_titles*`与tests共 **23/23** 个Rust文件、**798** 行已逐文件阅读。覆盖section identity、surface/theme/metrics、icon identity/geometry/manual glyph、label/strong-text commands及identity/theme/asset-contract/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，规模trace尚未完成，因此仍留在`pending.md`。

## P1：稳定标题重复theme/metrics和文字复制

带icon且font weight>=600的标题每paint约6次metrics和4次palette投影。`template_node_label`先复制完整label，strong rendering又为两层text command各`to_string`一次，累计3份owned String。Cube/Mesh/Transform手工icon分别展开6/6/4个quad。标题节点数量通常显著少于list/table/tree rows，因此优先级为P1，但稳定frame没有理由重复这些工作。

PERF-MVP-206要求changed title一次构建`SectionTitlePaintSpec`，包含typed icon/strong state、单一theme/metrics snapshot、shared label、surface/icon/text geometry与ordered commands；stable generation由PERF-MVP-178复用compiled segment。若产品trace证明icon常驻，按PERF-MVP-179收敛为单mask/atlas command；否则记录其有界预算。

## 动态验收

在1/100/10,000 titles、icon/no-icon、normal/strong/disabled/muted/declared tone上记录identity、theme/metrics acquisition、label/String bytes、surface/icon/text build与commands。changed title identity/theme/metrics/label各<=1且owned bytes只等于最终必要文本；stable全部及build=0。保持flat header theme、strong offset、icon identity/tone/geometry、clip/order和GPU/Softbuffer pixels parity。
